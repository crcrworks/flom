use std::fs;
use std::io::{self, IsTerminal, Read};

use clap::Parser;
use console::style;
use dialoguer::{Confirm, Input, Select, theme::ColorfulTheme};
use flom_config::{load_config, resolve_default_target, resolve_odesli_key, resolve_simple_output};
use flom_core::{ConversionResult, FlomError};
use flom_music::MusicConverter;
use flom_shorten::ShortenClient;

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
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

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
    if let Some(key) = resolve_odesli_key(config) {
        return Some(key);
    }

    let theme = ColorfulTheme::default();
    let input: String = Input::with_theme(&theme)
        .with_prompt("Odesli API key (optional, press Enter to skip)")
        .allow_empty(true)
        .interact_text()
        .unwrap_or_default();

    if input.trim().is_empty() {
        return None;
    }

    config.api.odesli_key = Some(input.clone());
    if Confirm::with_theme(&theme)
        .with_prompt("Save API key to ~/.flom/config.toml?")
        .default(true)
        .interact()
        .unwrap_or(false)
    {
        if let Err(err) = flom_config::save_config(config) {
            eprintln!("{} {err}", style("Warning:").yellow());
        }
    }

    Some(input)
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
