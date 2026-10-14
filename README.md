# Broxer

Opens every link you click in the browser your rules choose — e.g. YouTube in Zen, GitHub in Chrome. Broxer registers as your default browser, picks a browser in a few milliseconds and exits.

## Install

Download the installer for your OS from [Releases](https://github.com/dmbsk/broxer/releases/latest), or build it yourself with `cargo build --release` (Rust 1.88+).

**Windows** (`broxer-setup-windows-x64.exe`)
1. Run the installer. It installs per user (no admin), registers Broxer and adds **Broxer Settings** to the Start menu.
2. On the last page keep both boxes ticked: set up rules in Settings and **Save**, then choose Broxer as the web browser in Default apps.
3. Uninstall from *Settings → Apps*; this removes the registration too.

**macOS** (`broxer-macos-arm64.dmg`, Apple Silicon)
1. Open the dmg and drag both apps onto **Applications**.
2. The apps are unsigned: run `xattr -dr com.apple.quarantine /Applications/Broxer*.app` once.
3. Open Broxer Settings, set up rules, **Save**, **Register Broxer**, confirm the macOS prompt.

**Linux** (`broxer_<version>_amd64.deb`)
1. `sudo apt install ./broxer_<version>_amd64.deb`. Other distros: extract `broxer-linux-x64.tar.gz` into one directory (e.g. `~/.local/bin`).
2. Run Broxer Settings, set up rules, **Save**, **Register Broxer** (sets the default via `xdg-settings`).

Releasing: bump `version` in `Cargo.toml`, then push the matching `v<version>` tag; the Release workflow builds the installers and archives.

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
