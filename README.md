# Flom

Universal converter for music URLs and link shortening.

## Features

- Convert music URLs between streaming platforms (Spotify, Apple Music, YouTube Music, etc.)
- Shorten URLs using is.gd
- Interactive CLI with configuration support

## Installation

```bash
cargo install --path .
```

Or build from source:

```bash
cargo build --release
```

## Usage

### Music Conversion

Convert a Spotify link to Apple Music:

```bash
flom "https://open.spotify.com/track/example" --to apple-music
```

Convert without specifying target (interactive selection):

```bash
flom "https://music.apple.com/us/album/example"
```

### URL Shortening

```bash
flom "https://example.com/very/long/url" --shorten
```

### Configuration

Create/edit config file:

```bash
flom config edit
```

Configuration file location: `~/.flom/config.toml`

Example config:

```toml
[api]
odesli_key = "your-api-key-here"

[default]
target = "spotify"
user_country = "US"

[output]
simple = false
```

### Environment Variables

- `FLOM_ODESLI_KEY`: Odesli API key (overrides config file)
- `FLOM_DEFAULT_TARGET`: Default target platform (overrides config file)
- `FLOM_OUTPUT_SIMPLE`: Simple output mode (true/false/1/0)
- `FLOM_USER_COUNTRY`: User country code for platform availability (overrides config file, default: "US")

## Supported Platforms

- Spotify
- Apple Music
- iTunes
- YouTube
- YouTube Music
- Tidal
- Deezer
- Amazon Music

## What is NOT Included

- API key encryption (stored as plain text in config file)
- Extensive documentation (this README provides essential info)
- CI/CD pipelines
- Advanced error recovery

## License

MIT
