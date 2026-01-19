use regex::Regex;
use url::Url;

pub fn parse_apple_music_track_id(input: &str) -> Option<String> {
    let url = Url::parse(input).ok()?;
    if url.domain()? != "music.apple.com" {
        return None;
    }
    let query_pairs = url.query_pairs().collect::<Vec<_>>();
    if let Some((_, value)) = query_pairs.iter().find(|(key, _)| key == "i") {
        return Some(value.to_string());
    }

    let regex = Regex::new(r"music\.apple\.com/.*/(?:song|album)/.+/(\d+)").ok()?;
    let captures = regex.captures(input)?;
    captures.get(1).map(|m| m.as_str().to_string())
}

#[cfg(test)]
mod tests {
    use super::parse_apple_music_track_id;

    #[test]
    fn parses_apple_music_track_id_from_query() {
        let url = "https://music.apple.com/us/album/blinding-lights/1496794033?i=1496794038";
        assert_eq!(
            parse_apple_music_track_id(url),
            Some("1496794038".to_string())
        );
    }
}
