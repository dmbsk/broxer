#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."

cargo build --release -p broxer -p broxer-settings
version=$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)
root=target/deb/broxer
rm -rf "$root" && mkdir -p "$root/DEBIAN" "$root/usr/bin" "$root/usr/share/applications"

install -m755 target/release/broxer target/release/broxer-settings "$root/usr/bin/"
cat > "$root/usr/share/applications/broxer.desktop" <<DESKTOP
[Desktop Entry]
Type=Application
Name=Broxer
Comment=Opens each link in the browser your rules choose
Exec=/usr/bin/broxer %u
Terminal=false
NoDisplay=true
MimeType=x-scheme-handler/http;x-scheme-handler/https;
Categories=Network;WebBrowser;
DESKTOP
cat > "$root/usr/share/applications/broxer-settings.desktop" <<DESKTOP
[Desktop Entry]
Type=Application
Name=Broxer Settings
Comment=Choose which browser opens which links
Exec=/usr/bin/broxer-settings
Terminal=false
Categories=Network;Settings;
DESKTOP
cat > "$root/DEBIAN/control" <<CONTROL
Package: broxer
Version: $version
Architecture: amd64
Maintainer: dmbsk <https://github.com/dmbsk/broxer>
Depends: libc6, libxkbcommon0, libgl1, xdg-utils
Recommends: libnotify-bin
Section: web
Priority: optional
Description: Opens each link in the browser your rules choose
 Broxer registers as the default browser and hands every link to the
 browser picked by your domain rules, e.g. YouTube in Zen, GitHub in Chrome.
CONTROL

mkdir -p dist
dpkg-deb --build --root-owner-group "$root" "dist/broxer_${version}_amd64.deb"
