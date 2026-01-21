# Flom

Universal converter for anything. / ありとあらゆるものを変換できるツール。

## Philosophy / 思想

Flom is designed as an extensible converter framework. The vision is to handle any form of conversion through modular components.

Flomは拡張可能なコンバーターフレームワークとして設計されています。モジュール化されたコンポーネントを通じて、あらゆる形式の変換を扱うことを目指しています。

**ありとあらゆるものを変換できるツール**

## Features / 機能

- Extensible converter framework / 拡張可能なコンバーターフレームワーク
- Music URL conversion / 音楽URL変換 (Spotify, Apple Music, YouTube Music, etc.)
- URL shortening / URL短縮 (converter module example)
- Interactive CLI with configuration support / 対話的なCLIと設定サポート

## Architecture / アーキテクチャ

Flom is organized as modular workspace crates. Each crate handles a specific aspect of the converter framework.

Flomはワークスペートクレートとしてモジュール化されています。各クレートはコンバーターフレームワークの特定の側面を担当します。

### Core Modules / コアモジュール

- `flom-core`: Core utilities and types (エラー型、結果型、URL検証)
- `flom-config`: Configuration management (設定管理)

### Converter Modules / コンバータモジュール

- `flom-music`: Music URL converter module (音楽URLコンバータモジュール)
- `flom-shorten`: URL shortening converter module (URL短縮コンバータモジュール)

## Installation

```bash
cargo install --path .
```

Or build from source:

```bash
cargo build --release
```

## Usage / 使用方法

Current implementations include the following examples:
現在の実装には以下の例が含まれます：

### Example: Music URL Conversion / 例: 音楽URL変換

Convert a Spotify link to Apple Music:

```bash
flom "https://open.spotify.com/track/example" --to apple-music
```

Convert without specifying target (interactive selection):

```bash
flom "https://music.apple.com/us/album/example"
```

### Example: URL Shortening / 例: URL短縮

```bash
flom "https://example.com/very/long/url" --shorten
```

### Configuration / 設定

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

### Environment Variables / 環境変数

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
