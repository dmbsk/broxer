# Broxer — design

Date: 2026-10-14

## Goal

Broxer is a rule-based browser selector. It is registered as the OS default browser; every link clicked in another app goes to Broxer, which opens it in the browser chosen by user rules (e.g. YouTube → Zen, GitHub → Chrome). Routing must be imperceptible: ≤20 ms from router start to browser launch.

Platforms: Windows, macOS, Linux.

### In scope

- Links opened from other applications (OS default-browser handoff) for `http` and `https`.
- Rules editable in a TOML file and through a settings GUI that edits the same file.
- Registering/unregistering Broxer as a browser with the OS.

### Out of scope

- Links clicked inside a browser (would need a browser extension).
- Path or regex matching (domains only).
- A chooser popup for unmatched links.
- Installers, code signing, auto-update.

## Architecture

One Cargo workspace, private GitHub repo `dmbsk/broxer`:

```
crates/core/      config model, loading, validation, URL → browser matching. No OS calls.
crates/router/    `broxer` binary. Handles a link and exits.
crates/settings/  `broxer-settings` binary. egui editor + OS registration + browser detection.
```

The router never links GUI code. Both binaries use `core`, so matching behaviour is identical in the router and the settings "test URL" box.

## Config

Location (via the platform config dir):

| OS | Path |
|---|---|
| Windows | `%APPDATA%\broxer\config.toml` |
| macOS | `~/Library/Application Support/broxer/config.toml` |
| Linux | `~/.config/broxer/config.toml` |

Format:

```toml
fallback = "zen"

[browsers]
zen         = { path = 'C:\Program Files\Zen Browser\zen.exe' }
chrome      = { path = 'C:\Program Files\Google\Chrome\Application\chrome.exe' }
chrome-work = { path = 'C:\Program Files\Google\Chrome\Application\chrome.exe', args = ['--profile-directory=Profile 2'] }

[[rules]]
match = ["youtube.com", "youtu.be"]
browser = "zen"

[[rules]]
match = ["github.com"]
browser = "chrome"
```

- `path` is an executable; on macOS it may be a `.app` bundle.
- `args` is optional and is placed before the URL.
- Validation: `fallback` and every rule's `browser` must name an entry in `browsers`; every rule must have at least one domain.

## Matching

- Parse the URL; take its host, lowercased, without port.
- A rule domain `d` matches host `h` when `h == d` or `h` ends with `"." + d`. So `github.com` matches `gist.github.com` but not `notgithub.com`.
- Rules are checked in file order; first match wins. No match → `fallback`.
- Unparseable URL or non-http(s) scheme → `fallback`.

## Router flow

1. Receive URL (argv on Windows/Linux; Apple Event on macOS, which also accepts argv so tests and the CLI work the same everywhere).
2. Load and validate config, match the URL, launch the browser detached with `args + [url]`, exit without waiting.

Windows: built with the GUI subsystem so no console window flashes.

### Error handling

- **Never delegate to "the default browser"**: Broxer is the default, so that would loop. Always launch a concrete executable.
- Config missing or invalid → open the URL in the first detected installed browser (excluding Broxer) and show a notice describing the config error.
- Chosen browser's executable missing → use `fallback`; if that also fails, use the first detected browser. Show a notice.
- Notices: Windows message box, macOS alert, Linux `notify-send` (silently skipped if absent). All errors are also appended to `broxer.log` in the config directory.

## Platform integration

| | URL delivery | Registration | Launch |
|---|---|---|---|
| Windows | argv via `HKCU\Software\Classes\BroxerURL\shell\open\command` = `"<dir>\broxer.exe" "%1"` | Writes `BroxerURL` ProgID, `HKCU\Software\Clients\StartMenuInternet\Broxer` (Capabilities, URLAssociations `http`/`https` → `BroxerURL`) and `HKCU\Software\RegisteredApplications`. No admin. User then selects Broxer in Settings → Default apps (Windows forbids setting it programmatically); the settings app opens that page. | `CreateProcess` on the exe |
| macOS | `Broxer.app` declares `CFBundleURLTypes` for http/https and `LSUIElement` (no Dock icon). Router registers a `kAEGetURL` handler, runs the event loop until the URL arrives (short timeout), routes, and quits. | `LSSetDefaultHandlerForURLScheme` for http and https; macOS shows its own confirmation. | `NSWorkspace openURLs:withApplicationAtURL:configuration:` (args passed via configuration) |
| Linux | argv `%u` from `~/.local/share/applications/broxer.desktop` (`MimeType=x-scheme-handler/http;x-scheme-handler/https;`) | Writes the `.desktop` file, runs `xdg-settings set default-web-browser broxer.desktop`. | Spawn the exe |

Unregister removes everything that register wrote.

### Browser detection (first run and error fallback)

- Windows: `StartMenuInternet` entries under HKLM and HKCU.
- macOS: `LSCopyApplicationURLsForURL` for an `https` URL.
- Linux: `.desktop` files in XDG data dirs advertising `x-scheme-handler/https`.

Broxer itself is excluded from detected browsers.

## Settings app

Single egui window:

- **Browsers**: table of name, path (with Browse), args. Pre-filled from detection when no config exists.
- **Rules**: ordered list; each row = comma-separated domains + browser dropdown; add, delete, move up/down.
- **Fallback**: browser dropdown.
- **Test URL**: shows which browser a pasted URL would open in, using `core`.
- **Save**: validates, then writes the TOML. Hand-written comments in the file are not preserved.
- **Register / Unregister / Open default-apps settings** buttons (last one Windows only; on macOS/Linux registration sets the default directly).

## Packaging

- Windows: `broxer.exe`, `broxer-settings.exe`; README instructs copying to `%LOCALAPPDATA%\Programs\broxer\` then clicking Register.
- macOS: `Broxer.app` (router) and `Broxer Settings.app`, produced by a bundling script; unsigned.
- Linux: two binaries; the `.desktop` file is generated by Register.
- Release profile: LTO, `opt-level = 3`, `panic = "abort"`, `codegen-units = 1`.
- Router dependencies limited to `serde`, `toml`, `url`, a config-dir helper, and minimal OS bindings (no async runtime, no GUI).

## Testing

- `core` unit tests: exact/subdomain/lookalike matching, `www.`, case, ports, first-match-wins, fallback, malformed URL, non-http scheme, config validation errors, missing file.
- Router integration test: run the real `broxer` binary with a temp config pointing at a fake-browser test binary that records its argv; assert the right browser and args received the URL. Platform glue (Apple Events, registry) sits behind small functions so routing is testable without it.
- Speed test: 100 router runs against the fake browser; fail if median exceeds 20 ms.
- CI: GitHub Actions on Windows, macOS, Ubuntu — `cargo test`, `cargo clippy -- -D warnings`, release build.
- Manual (README checklist): register, set default, click a YouTube and a GitHub link from another app on each OS. macOS and Linux end-to-end link handling are verified manually by the user.

## Amendments (from planning, 2026-10-14)

- A fourth crate, `crates/platform`, holds OS code shared by both binaries (config path, launching, browser detection, notices). `core` stays OS-free; registration lives in `settings`.
- macOS launch uses `/usr/bin/open -a <App.app> <url> [--args ...]` instead of NSWorkspace: NSWorkspace launches asynchronously and the router would have to wait for the completion handler before exiting, which costs more than it saves.
- macOS browser detection scans `/Applications` and `~/Applications` for bundles whose `Info.plist` lists `https` in `CFBundleURLTypes`, instead of calling `LSCopyApplicationURLsForURL` (avoids CoreServices FFI in the router).
- macOS "Unregister" cannot unset a default browser; it tells the user to pick another browser in System Settings.
- Environment overrides: `BROXER_CONFIG` replaces the config path; `BROXER_QUIET` suppresses popups (log only). Both exist for tests and scripting.
- Starting the router with no URL opens Broxer Settings (macOS: after a 3 s wait for a URL Apple Event).
- Rule domains are normalized on load: scheme, path, port, `*.` prefix, case, and trailing dot are stripped; internationalized names become punycode. A UTF-8 BOM at the start of the config file is ignored.
