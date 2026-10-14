# Broxer

Opens every link you click in the browser your rules choose — e.g. YouTube in Zen, GitHub in Chrome. Broxer registers as your default browser, picks a browser in a few milliseconds and exits.

## Install

Build: `cargo build --release` (Rust 1.88+).

**Windows**
1. Copy `target/release/broxer.exe` and `broxer-settings.exe` to `%LOCALAPPDATA%\Programs\broxer\`.
2. Run `broxer-settings.exe`, set up rules, **Save**, **Register Broxer**.
3. Click **Open Default apps** and choose Broxer as the web browser.

**macOS**
1. `scripts/bundle-macos.sh`, then move `dist/Broxer.app` and `dist/Broxer Settings.app` to `/Applications`.
2. Open Broxer Settings, set up rules, **Save**, **Register Broxer**, confirm the macOS prompt.

**Linux**
1. Copy `target/release/broxer` and `broxer-settings` into the same directory (e.g. `~/.local/bin`).
2. Run `broxer-settings`, set up rules, **Save**, **Register Broxer** (sets the default via `xdg-settings`).

## Config

Broxer Settings edits this file; you can also edit it by hand (comments are lost when Settings saves).

| OS | Path |
|---|---|
| Windows | `%APPDATA%\broxer\config.toml` |
| macOS | `~/Library/Application Support/broxer/config.toml` |
| Linux | `~/.config/broxer/config.toml` |

```toml
fallback = "zen"

[browsers]
zen = { path = 'C:\Program Files\Zen Browser\zen.exe' }
chrome = { path = 'C:\Program Files\Google\Chrome\Application\chrome.exe' }
chrome-work = { path = 'C:\Program Files\Google\Chrome\Application\chrome.exe', args = ["--profile-directory=Profile 2"] }

[[rules]]
match = ["youtube.com", "youtu.be"]
browser = "zen"

[[rules]]
match = ["github.com"]
browser = "chrome"
```

- A domain matches itself and its subdomains (`github.com` matches `gist.github.com`, not `notgithub.com`).
- Rules are checked top to bottom; the first match wins. Unmatched links go to `fallback`.
- On macOS `path` may be an app bundle, e.g. `/Applications/Zen.app`.
- Problems (bad config, missing browser) are shown in a popup and written to `broxer.log` next to the config. The link still opens in the fallback or another installed browser.
- `BROXER_CONFIG` overrides the config path; `BROXER_QUIET=1` suppresses popups.

## Manual check (each OS)

1. Register Broxer and set it as the default browser.
2. From another app (chat, email, terminal), click a YouTube link → opens in Zen.
3. Click a GitHub link → opens in Chrome.
4. Click any other link → opens in the fallback.
5. Break the config (e.g. delete a browser referenced by a rule) → link still opens, a popup explains the problem.
