use std::collections::HashMap;

use flom_core::{ConversionResult, FlomError, FlomResult, MediaInfo};
use reqwest::Client;
use url::Url;

use crate::api::odesli::{OdesliClient, OdesliResponse};

#[derive(Debug, Clone)]
pub struct TargetOption {
    pub key: String,
    pub label: String,
}

#[derive(Debug, Clone)]
pub struct MusicConverter {
    client: OdesliClient,
}

impl MusicConverter {
    pub fn new(api_key: Option<String>) -> Self {
        let client = Client::builder()
            .user_agent("flom/0.1")
            .build()
            .expect("failed to build http client");
        Self {
            client: OdesliClient::new(client, api_key, "US"),
        }
    }

    pub async fn fetch_links(&self, url: &str) -> FlomResult<OdesliResponse> {
        validate_url(url)?;
        self.client.fetch_links(url).await
    }

    pub fn targets_from_response(response: &OdesliResponse) -> Vec<TargetOption> {
        response
            .links_by_platform
            .keys()
            .map(|key| TargetOption {
                key: key.clone(),
                label: display_name(key).to_string(),
            })
            .collect()
    }

    pub fn normalize_target(input: &str) -> Option<String> {
        let normalized = input.trim().to_lowercase();
        match normalized.as_str() {
            "spotify" => Some("spotify".to_string()),
            "applemusic" | "apple-music" | "apple_music" => Some("appleMusic".to_string()),
            "itunes" => Some("itunes".to_string()),
            "youtube" => Some("youtube".to_string()),
            "youtubemusic" | "youtube-music" | "youtube_music" => Some("youtubeMusic".to_string()),
            "tidal" => Some("tidal".to_string()),
            "deezer" => Some("deezer".to_string()),
            "amazonmusic" | "amazon-music" | "amazon_music" => Some("amazonMusic".to_string()),
            _ => None,
        }
    }

    pub fn convert_from_response(
        response: &OdesliResponse,
        source_url: &str,
        target_key: &str,
    ) -> FlomResult<ConversionResult> {
        let source_entity = response
            .entities_by_unique_id
            .get(&response.entity_unique_id);

        let source_info = source_entity.map(entity_to_media);
        let source_platform = source_entity
            .and_then(|entity| entity.api_provider.clone())
            .or_else(|| infer_source_platform(&response.links_by_platform, source_url));

        let target_link = response.links_by_platform.get(target_key).ok_or_else(|| {
            FlomError::UnsupportedInput(format!("target platform not available: {target_key}"))
        })?;

        let target_entity = response
            .entities_by_unique_id
            .get(&target_link.entity_unique_id);

        Ok(ConversionResult {
            source_url: source_url.to_string(),
            target_url: Some(target_link.url.clone()),
            source_platform,
            target_platform: Some(target_key.to_string()),
            source_info,
            target_info: target_entity.map(entity_to_media),
            warning: None,
        })
    }
}

fn validate_url(url: &str) -> FlomResult<()> {
    Url::parse(url).map_err(|err| FlomError::InvalidInput(format!("invalid url: {err}")))?;
    Ok(())
}

fn display_name(key: &str) -> &str {
    match key {
        "appleMusic" => "Apple Music",
        "itunes" => "iTunes",
        "spotify" => "Spotify",
        "youtube" => "YouTube",
        "youtubeMusic" => "YouTube Music",
        "tidal" => "Tidal",
        "deezer" => "Deezer",
        "amazonMusic" => "Amazon Music",
        _ => key,
    }
}

fn entity_to_media(entity: &crate::api::odesli::OdesliEntity) -> MediaInfo {
    MediaInfo {
        title: entity.title.clone(),
        artist: entity.artist_name.clone(),
        album: entity.album_name.clone(),
    }
}

fn infer_source_platform(
    links: &HashMap<String, crate::api::odesli::OdesliLink>,
    url: &str,
) -> Option<String> {
    links
        .iter()
        .find(|(_, link)| link.url == url)
        .map(|(key, _)| key.clone())
}

#[cfg(test)]
mod tests {
    use super::MusicConverter;

    #[test]
    fn normalize_target_maps_common_inputs() {
        assert_eq!(
            MusicConverter::normalize_target("spotify"),
            Some("spotify".to_string())
        );
        assert_eq!(
            MusicConverter::normalize_target("apple-music"),
            Some("appleMusic".to_string())
        );
        assert_eq!(
            MusicConverter::normalize_target("youtube_music"),
            Some("youtubeMusic".to_string())
        );
    }
}
