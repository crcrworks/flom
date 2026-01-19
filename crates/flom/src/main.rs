use std::fs;
use std::io::{self, IsTerminal, Read};

use clap::{Parser, Subcommand};
use console::style;
use dialoguer::{Input, Select, theme::ColorfulTheme};
use flom_config::{
    config_exists, load_config, open_in_editor, resolve_default_target, resolve_simple_output,
    save_config, set_config_value,
};
use flom_core::{ConversionResult, FlomError, FlomResult};
use flom_music::MusicConverter;
use flom_shorten::ShortenClient;

#[derive(Subcommand, Debug)]
enum Commands {
    /// Manage configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand, Debug)]
enum ConfigAction {
    /// Get a configuration value
    Get { key: String },
    /// Set a configuration value
    Set { key: String, value: String },
    /// List all configuration values
    List,
    /// Open config file in editor
    Edit,
}

#[derive(Debug, Parser)]
#[command(name = "flom")]
#[command(version, about = "Universal converter", long_about = None)]
struct Cli {
    #[arg(long)]
    to: Option<String>,
    #[arg(long)]
    input: Option<String>,
    #[arg(long)]
    shorten: bool,
    #[arg(long)]
    simple: bool,
    #[arg(value_name = "URL")]
    urls: Vec<String>,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Handle config commands first
    if let Some(Commands::Config { action }) = cli.command {
        if let Err(err) = handle_config_command(action) {
            eprintln!("{} {err}", style("Error:").red());
            std::process::exit(1);
        }
        return;
    }

    let mut config = match load_config() {
        Ok(config) => config,
        Err(err) => {
            eprintln!("{} {err}", style("Error:").red());
            std::process::exit(1);
        }
    };

    let mut urls = gather_inputs(&cli).unwrap_or_else(|err| {
        eprintln!("{} {err}", style("Error:").red());
        std::process::exit(1);
    });

    if urls.is_empty() {
        eprintln!("{} no input URLs provided", style("Error:").red());
        std::process::exit(1);
    }

    if cli.shorten {
        run_shorten(&urls).await;
        return;
    }

    let api_key = resolve_or_prompt_odesli_key(&mut config);
    let converter = MusicConverter::new(api_key);

    let simple = cli.simple || resolve_simple_output(&config).unwrap_or(false);
    let default_target = resolve_default_target(&config);

    let mut success = 0usize;
    let mut failed = 0usize;

    for url in urls.drain(..) {
        match process_url(
            &converter,
            &url,
            cli.to.as_deref(),
            default_target.as_deref(),
            simple,
        )
        .await
        {
            Ok(count) => success += count,
            Err(err) => {
                failed += 1;
                eprintln!("{} {url}: {err}", style("Failed").red());
            }
        }
    }

    print_summary(success + failed, success, failed);
}

fn gather_inputs(cli: &Cli) -> Result<Vec<String>, FlomError> {
    let mut urls = cli.urls.clone();

    if let Some(path) = &cli.input {
        let content = fs::read_to_string(path)
            .map_err(|err| FlomError::InvalidInput(format!("failed to read input file: {err}")))?;
        urls.extend(parse_lines(&content));
    }

    if urls.is_empty() && !io::stdin().is_terminal() {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .map_err(|err| FlomError::InvalidInput(format!("failed to read stdin: {err}")))?;
        urls.extend(parse_lines(&buffer));
    }

    Ok(urls)
}

fn parse_lines(content: &str) -> Vec<String> {
    content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| line.to_string())
        .collect()
}

fn resolve_or_prompt_odesli_key(config: &mut flom_config::FlomConfigData) -> Option<String> {
    // Check environment variable first
    if let Ok(value) = std::env::var("FLOM_ODESLI_KEY") {
        if !value.trim().is_empty() {
            return Some(value);
        }
    }

    // If config file exists, use its value (never prompt)
    if config_exists().unwrap_or(false) {
        return config.api.odesli_key.clone();
    }

    // Config file doesn't exist - first time setup
    let theme = ColorfulTheme::default();
    println!(
        "{} {}",
        style("First-time setup:").bold().cyan(),
        "Let's configure your flom settings"
    );

    let input: String = Input::with_theme(&theme)
        .with_prompt("Odesli API key (optional, press Enter to skip)")
        .allow_empty(true)
        .interact_text()
        .unwrap_or_default();

    if !input.trim().is_empty() {
        config.api.odesli_key = Some(input.clone());
    }

    // Always create config file on first run
    if let Err(err) = save_config(config) {
        eprintln!("{} {err}", style("Warning:").yellow());
    } else {
        println!(
            "{} Config file created at ~/.flom/config.toml",
            style("✓").green()
        );
    }

    config.api.odesli_key.clone()
}

fn handle_config_command(action: ConfigAction) -> FlomResult<()> {
    match action {
        ConfigAction::Get { key } => {
            let config = load_config()?;
            let value = get_nested_config_value(&config, &key);
            match value {
                Some(v) => println!("{} = {}", key, v),
                None => println!("{} = <null>", key),
            }
            Ok(())
        }
        ConfigAction::Set { key, value } => {
            set_config_value(&key, &value)?;
            println!("{} Set {} = {}", style("✓").green(), key, value);
            Ok(())
        }
        ConfigAction::List => {
            let config = load_config()?;
            println!("Current configuration:");
            println!("\n[api]");
            println!(
                "odesli_key = {}",
                config.api.odesli_key.as_deref().unwrap_or("<null>")
            );
            println!("\n[default]");
            println!(
                "target = {}",
                config.default.target.as_deref().unwrap_or("<null>")
            );
            println!("\n[output]");
            println!("simple = {}", config.output.simple.unwrap_or(false));
            Ok(())
        }
        ConfigAction::Edit => {
            open_in_editor()?;
            Ok(())
        }
    }
}

fn get_nested_config_value(config: &flom_config::FlomConfigData, key_path: &str) -> Option<String> {
    let parts: Vec<&str> = key_path.split('.').collect();

    match parts.as_slice() {
        ["api", "odesli_key"] => config.api.odesli_key.clone(),
        ["default", "target"] => config.default.target.clone(),
        ["output", "simple"] => config.output.simple.map(|b| b.to_string()),
        _ => None,
    }
}

async fn process_url(
    converter: &MusicConverter,
    url: &str,
    explicit_target: Option<&str>,
    default_target: Option<&str>,
    simple: bool,
) -> Result<usize, FlomError> {
    let response = converter.fetch_links(url).await?;
    let target = explicit_target
        .map(|value| value.to_string())
        .or_else(|| default_target.map(|value| value.to_string()));

    let target_key = if let Some(target) = target {
        let normalized = target.trim().to_lowercase();
        if normalized == "all" {
            "all".to_string()
        } else if normalized == "songlink" {
            "songlink".to_string()
        } else {
            MusicConverter::normalize_target(&target)
                .ok_or_else(|| FlomError::InvalidInput(format!("unknown target: {target}")))?
        }
    } else {
        prompt_target(&response)?
    };

    if target_key == "all" {
        let mut count = 0;
        let mut keys: Vec<_> = response.links_by_platform.keys().cloned().collect();
        keys.sort();
        for key in keys {
            let result = MusicConverter::convert_from_response(&response, url, &key)?;
            print_result(&result, simple);
            count += 1;
        }
        return Ok(count);
    }

    if target_key == "songlink" {
        let result = ConversionResult {
            source_url: url.to_string(),
            target_url: Some(response.page_url.clone()),
            source_platform: None,
            target_platform: Some("songlink".to_string()),
            source_info: None,
            target_info: None,
            warning: None,
        };
        print_result(&result, simple);
        return Ok(1);
    }

    let result = MusicConverter::convert_from_response(&response, url, &target_key)?;
    print_result(&result, simple);
    Ok(1)
}

fn prompt_target(response: &flom_music::api::odesli::OdesliResponse) -> Result<String, FlomError> {
    let mut options = MusicConverter::targets_from_response(response);
    options.sort_by(|a, b| a.label.cmp(&b.label));

    let mut labels: Vec<String> = options.iter().map(|opt| opt.label.clone()).collect();
    labels.push("All available".to_string());
    labels.push("Songlink page".to_string());

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select target platform")
        .items(&labels)
        .default(0)
        .interact()
        .map_err(|err| FlomError::InvalidInput(format!("selection failed: {err}")))?;

    if selection == labels.len() - 2 {
        return Ok("all".to_string());
    }
    if selection == labels.len() - 1 {
        return Ok("songlink".to_string());
    }

    Ok(options[selection].key.clone())
}

fn print_result(result: &ConversionResult, simple: bool) {
    if simple {
        if let Some(url) = &result.target_url {
            println!("{url}");
        }
        return;
    }

    let source_line = format_source_line(result);
    println!("{} {source_line}", style("From:").cyan());
    println!("  {} {}", style("URL:").dim(), result.source_url);

    if let Some(target_url) = &result.target_url {
        println!("{} {}", style("To:").green(), target_url);
    } else {
        println!("{} (no target url)", style("To:").red());
    }

    if let Some(warning) = &result.warning {
        println!("{} {warning}", style("Warning:").yellow());
    }

    println!();
}

fn format_source_line(result: &ConversionResult) -> String {
    let platform = result.source_platform.as_deref().unwrap_or("Unknown");
    if let Some(info) = &result.source_info {
        let title = info.title.as_deref().unwrap_or("Unknown title");
        let artist = info.artist.as_deref().unwrap_or("Unknown artist");
        return format!("{platform} - {title} / {artist}");
    }
    platform.to_string()
}

async fn run_shorten(urls: &[String]) {
    let client = ShortenClient::new();
    let mut success = 0usize;
    let mut failed = 0usize;

    for url in urls {
        match client.shorten(url).await {
            Ok(short) => {
                println!("{} -> {}", url, short);
                success += 1;
            }
            Err(err) => {
                failed += 1;
                eprintln!("{} {url}: {err}", style("Failed").red());
            }
        }
    }

    print_summary(success + failed, success, failed);
}

fn print_summary(total: usize, success: usize, failed: usize) {
    println!(
        "{} Total: {} | Success: {} | Failed: {}",
        style("Summary:").bold(),
        total,
        success,
        failed
    );
}
