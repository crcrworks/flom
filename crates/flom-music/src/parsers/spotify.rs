use regex::Regex;

pub fn parse_spotify_track_id(input: &str) -> Option<String> {
    let regex = Regex::new(r"open\.spotify\.com/(?:intl-[a-z]{2}/)?track/([A-Za-z0-9]+)").ok()?;
    let captures = regex.captures(input)?;
    captures.get(1).map(|m| m.as_str().to_string())
}

#[cfg(test)]
mod tests {
    use super::parse_spotify_track_id;

    #[test]
    fn parses_spotify_track_id() {
        let url = "https://open.spotify.com/track/4Km5HrUvYTaSUfiSGPJeQR";
        assert_eq!(
            parse_spotify_track_id(url),
            Some("4Km5HrUvYTaSUfiSGPJeQR".to_string())
        );
    }

    #[test]
    fn parses_spotify_track_id_with_locale() {
        let url = "https://open.spotify.com/intl-ja/track/4Km5HrUvYTaSUfiSGPJeQR";
        assert_eq!(
            parse_spotify_track_id(url),
            Some("4Km5HrUvYTaSUfiSGPJeQR".to_string())
        );
    }
}
