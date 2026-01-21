use std::collections::HashMap;

use flom_config::{FlomConfigData, resolve_user_country};
use flom_core::{ConversionResult, FlomError, FlomResult, MediaInfo, validate_url};
use reqwest::Client;

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
    pub fn new(api_key: Option<String>, config: &FlomConfigData) -> Self {
        let client = Client::builder()
            .user_agent("flom/0.1")
            .build()
            .expect("failed to build http client");
        let user_country = resolve_user_country(config);
        Self {
            client: OdesliClient::new(client, api_key, user_country),
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
    use super::*;

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
        assert_eq!(
            MusicConverter::normalize_target("  AMAZON_MUSIC  "),
            Some("amazonMusic".to_string())
        );
        assert_eq!(
            MusicConverter::normalize_target("YouTubeMusic"),
            Some("youtubeMusic".to_string())
        );
    }

    #[test]
    fn test_normalize_target_undefined() {
        assert_eq!(MusicConverter::normalize_target("unknown"), None);
        assert_eq!(MusicConverter::normalize_target("not-a-platform"), None);
        assert_eq!(MusicConverter::normalize_target(""), None);
    }

    #[test]
    fn test_display_name_all_platforms() {
        // Test through targets_from_response
        let mut response = OdesliResponse {
            entity_unique_id: "test-id".to_string(),
            page_url: "https://example.com".to_string(),
            links_by_platform: HashMap::new(),
            entities_by_unique_id: HashMap::new(),
        };

        response.links_by_platform.insert(
            "appleMusic".to_string(),
            crate::api::odesli::OdesliLink {
                entity_unique_id: "id1".to_string(),
                url: "https://music.apple.com".to_string(),
            },
        );
        response.links_by_platform.insert(
            "itunes".to_string(),
            crate::api::odesli::OdesliLink {
                entity_unique_id: "id2".to_string(),
                url: "https://itunes.apple.com".to_string(),
            },
        );
        response.links_by_platform.insert(
            "spotify".to_string(),
            crate::api::odesli::OdesliLink {
                entity_unique_id: "id3".to_string(),
                url: "https://spotify.com".to_string(),
            },
        );
        response.links_by_platform.insert(
            "youtube".to_string(),
            crate::api::odesli::OdesliLink {
                entity_unique_id: "id4".to_string(),
                url: "https://youtube.com".to_string(),
            },
        );
        response.links_by_platform.insert(
            "youtubeMusic".to_string(),
            crate::api::odesli::OdesliLink {
                entity_unique_id: "id5".to_string(),
                url: "https://music.youtube.com".to_string(),
            },
        );
        response.links_by_platform.insert(
            "tidal".to_string(),
            crate::api::odesli::OdesliLink {
                entity_unique_id: "id6".to_string(),
                url: "https://tidal.com".to_string(),
            },
        );
        response.links_by_platform.insert(
            "deezer".to_string(),
            crate::api::odesli::OdesliLink {
                entity_unique_id: "id7".to_string(),
                url: "https://deezer.com".to_string(),
            },
        );
        response.links_by_platform.insert(
            "amazonMusic".to_string(),
            crate::api::odesli::OdesliLink {
                entity_unique_id: "id8".to_string(),
                url: "https://music.amazon.com".to_string(),
            },
        );

        let targets = MusicConverter::targets_from_response(&response);
        assert!(
            targets
                .iter()
                .any(|t| t.key == "appleMusic" && t.label == "Apple Music")
        );
        assert!(
            targets
                .iter()
                .any(|t| t.key == "itunes" && t.label == "iTunes")
        );
        assert!(
            targets
                .iter()
                .any(|t| t.key == "spotify" && t.label == "Spotify")
        );
        assert!(
            targets
                .iter()
                .any(|t| t.key == "youtube" && t.label == "YouTube")
        );
        assert!(
            targets
                .iter()
                .any(|t| t.key == "youtubeMusic" && t.label == "YouTube Music")
        );
        assert!(
            targets
                .iter()
                .any(|t| t.key == "tidal" && t.label == "Tidal")
        );
        assert!(
            targets
                .iter()
                .any(|t| t.key == "deezer" && t.label == "Deezer")
        );
        assert!(
            targets
                .iter()
                .any(|t| t.key == "amazonMusic" && t.label == "Amazon Music")
        );
    }

    #[test]
    fn test_entity_to_media_full() {
        // Test through convert_from_response
        let mut response = OdesliResponse {
            entity_unique_id: "source-id".to_string(),
            page_url: "https://example.com".to_string(),
            links_by_platform: HashMap::new(),
            entities_by_unique_id: HashMap::new(),
        };

        response.entities_by_unique_id.insert(
            "source-id".to_string(),
            crate::api::odesli::OdesliEntity {
                id: Some("id1".to_string()),
                title: Some("Test Song".to_string()),
                artist_name: Some("Test Artist".to_string()),
                album_name: Some("Test Album".to_string()),
                api_provider: Some("spotify".to_string()),
            },
        );

        response.links_by_platform.insert(
            "spotify".to_string(),
            crate::api::odesli::OdesliLink {
                entity_unique_id: "source-id".to_string(),
                url: "https://spotify.com".to_string(),
            },
        );

        let result =
            MusicConverter::convert_from_response(&response, "https://spotify.com", "spotify");
        assert!(result.is_ok());
        let conversion_result = result.unwrap();
        assert_eq!(
            conversion_result.source_info,
            Some(MediaInfo {
                title: Some("Test Song".to_string()),
                artist: Some("Test Artist".to_string()),
                album: Some("Test Album".to_string()),
            })
        );
    }

    #[test]
    fn test_entity_to_media_partial() {
        // Test through convert_from_response with partial entity
        let mut response = OdesliResponse {
            entity_unique_id: "source-id".to_string(),
            page_url: "https://example.com".to_string(),
            links_by_platform: HashMap::new(),
            entities_by_unique_id: HashMap::new(),
        };

        response.entities_by_unique_id.insert(
            "source-id".to_string(),
            crate::api::odesli::OdesliEntity {
                id: None,
                title: Some("Test Song".to_string()),
                artist_name: Some("Test Artist".to_string()),
                album_name: None,
                api_provider: Some("spotify".to_string()),
            },
        );

        response.links_by_platform.insert(
            "spotify".to_string(),
            crate::api::odesli::OdesliLink {
                entity_unique_id: "source-id".to_string(),
                url: "https://spotify.com".to_string(),
            },
        );

        let result =
            MusicConverter::convert_from_response(&response, "https://spotify.com", "spotify");
        assert!(result.is_ok());
        let conversion_result = result.unwrap();
        assert_eq!(
            conversion_result.source_info,
            Some(MediaInfo {
                title: Some("Test Song".to_string()),
                artist: Some("Test Artist".to_string()),
                album: None,
            })
        );
    }

    #[test]
    fn test_validate_url_https() {
        assert!(validate_url("https://example.com").is_ok());
        assert!(validate_url("https://spotify.com/track/123").is_ok());
    }

    #[test]
    fn test_validate_url_http() {
        assert!(validate_url("http://example.com").is_ok());
        assert!(validate_url("http://music.example.com/album/456").is_ok());
    }

    #[test]
    fn test_validate_url_invalid() {
        let result = validate_url("not-a-url");
        assert!(result.is_err());
        match result {
            Err(FlomError::InvalidInput(msg)) => assert!(msg.contains("invalid url")),
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[test]
    fn test_validate_url_no_scheme() {
        let result = validate_url("://no-scheme");
        assert!(result.is_err());
        match result {
            Err(FlomError::InvalidInput(msg)) => assert!(msg.contains("invalid url")),
            _ => panic!("Expected InvalidInput error"),
        }
    }
}
