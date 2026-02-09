# Introducing ttv: A Terminal-Based Twitch Viewer

If you're like me, you probably have a handful of streamers you follow on Twitch. You're not browsing for new content every day—you just want to check if your favorite creators are live and maybe catch up on VODs when you have time. But here's the thing: the Twitch website can be painfully slow and overwhelming. It's designed for stream discovery, not for people who already know exactly what they want to watch.

That's why I built **ttv**, a lightweight command-line tool for watching Twitch streams and VODs from your terminal.

## The Problem

The Twitch web interface is a full-featured platform with recommendations, chat overlays, and endless navigation options. But if you just want to watch a stream on a second monitor while you work, all that friction adds up. Opening a browser, waiting for the page to load, navigating through menus, and dealing with a resource-heavy tab running in the background—it's overkill.

I wanted something simpler: a way to check who's online, pick a stream, and launch it in a lightweight video player. And since I spend most of my day in the terminal anyway, a CLI tool felt like the natural solution.

## What is ttv?

**ttv** is a Rust-based command-line application that lets you manage and watch Twitch streams directly from your terminal. It uses the Twitch API to fetch stream information and integrates with `streamlink` and `mpv` to play streams in a minimal video player—no browser required.

Here's what makes it different:

- **Local-only follows**: ttv maintains its own list of streamers you follow, stored in a local SQLite database. This doesn't sync with your Twitch account, which means you can manage a separate list optimized for terminal viewing without affecting your web follows.
- **Fast and focused**: No web bloat. Just quick commands to see who's live and start watching.
- **Terminal-first workflow**: Everything happens in your terminal, so you can stay in your flow without context-switching to a browser.

## Key Features

### Follow and manage streamers

```bash
# Follow one or more streamers
ttv follow shroud lirik

# List who's currently online
ttv list

# List all followed streamers (online and offline)
ttv list --status all

# Unfollow a streamer
ttv unfollow shroud
```

### Watch live streams

```bash
# Watch a specific streamer
ttv watch lirik

# No arguments? ttv shows you all online streamers to choose from
ttv watch
```

When you run `ttv watch`, it launches the stream using `streamlink` and `mpv`, giving you a clean, lightweight video player without the overhead of a browser.

### Watch VODs

```bash
# View recent VODs for a streamer
ttv vod shroud

# ttv will list available VODs and let you select one to watch
```

## How It Works

Under the hood, ttv uses the Twitch API to fetch streamer information and stream status. You'll need to set up a Twitch application (free and takes just a minute) to get a client ID and secret:

```bash
# Configure your Twitch API credentials
ttv config --client-id YOUR_CLIENT_ID --client-secret YOUR_CLIENT_SECRET

# Authenticate (fetches an access token)
ttv auth
```

Once configured, ttv handles authentication automatically and stores everything locally. Your follow list lives in a SQLite database in your home directory, and stream playback is handled by `streamlink` (which you'll need to install separately).

## Why I Built This

I wanted a tool that matched how I actually watch Twitch: casually checking if specific streamers are live, launching a stream on a second monitor, and getting back to work. The terminal is where I'm most productive, and ttv fits naturally into that workflow.

If you're comfortable with command-line tools and you're tired of the Twitch website's bloat, give ttv a try. It's open source and available on [GitHub](https://github.com/vstegen/ttv).

---

**About the Author**: I'm Marvin, a developer who spends too much time in the terminal and occasionally watches Twitch streams while coding. You can find more of my projects on GitHub.
