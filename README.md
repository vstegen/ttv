# ttv.rs

`ttv` is a small CLI tool for watching Twitch streams and VODs from the terminal. It integrates with the Twitch API to manage follows, list online channels, and launch playback via `streamlink` and `mpv`.

## Features

- Manage Twitch API credentials and access tokens
- Follow and unfollow streamers locally (stored in SQLite)
- List followed streamers with online/offline filtering and game info
- Watch live streams or select VODs to play

## Requirements

- Rust (edition 2024) toolchain
- `streamlink` installed and available on `PATH`
- `mpv` installed and available on `PATH`

## Install

From source:

```bash
git clone git@github.com:vstegen/ttv.git
cd ttv-rs
cargo build --release
```

The binary will be at `target/release/ttv-rs` (you may want to rename it to `ttv` or add it to your `PATH`).

## Configuration

`ttv` stores its configuration in the XDG config directory:

- Linux/macOS: `~/.config/ttv/config.json`
- Windows: `%APPDATA%\ttv\config.json`

Set your Twitch API credentials:

```bash
ttv config --client-id <CLIENT_ID> --client-secret <CLIENT_SECRET>
```

Fetch a new app access token:

```bash
ttv auth
```

## Usage

General help:

```bash
ttv --help
```

### Command Overview

```text
ttv config [--client-id <ID>] [--client-secret <SECRET>] [--access-token <TOKEN>] [--expires-at <RFC3339>] [--show]
ttv auth [--show] [--verbose]
ttv follow [--verbose] <LOGIN...>
ttv list [--status <online|offline|all>]
ttv watch <STREAM...>
ttv vod <LOGIN>
ttv unfollow [--verbose] <LOGIN...>
```

### Follow

Follow streamers locally (stored in SQLite):

```bash
ttv follow jonhoo theprimeagen
```

### List

List followed streamers (default: online only):

```bash
ttv list
ttv list --status all
ttv list --status offline
```

### Watch

Watch one or more live streams by login or URL:

```bash
ttv watch theprimeagen
ttv watch https://www.twitch.tv/jonhoo
```

### VOD

Select and watch a VOD for a streamer:

```bash
ttv vod theprimeagen
```

### Unfollow

Remove local follows:

```bash
ttv unfollow theprimeagen jonhoo
```

## Data Storage

`ttv` stores follows in a local SQLite database:

- Linux/macOS: `~/.local/share/ttv/ttv.sqlite`
- Windows: `%APPDATA%\ttv\ttv.sqlite`

## Troubleshooting

- If Twitch API requests fail, ensure you have a valid client ID/secret and run `ttv auth`.
- If playback fails, verify that `streamlink` and `mpv` are installed and on your `PATH`.
- Twitch ads can cause a black screen during playback. This is a known limitation of Twitch and `streamlink`, not `ttv`.
