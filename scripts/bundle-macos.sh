#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

cargo build --release -p broxer -p broxer-settings
version=$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)
rm -rf dist && mkdir -p dist

make_app() { # name exe bundle_id extra_plist
  local app="dist/$1.app"
  mkdir -p "$app/Contents/MacOS"
  cp "target/release/$2" "$app/Contents/MacOS/$2"
  cat > "$app/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>$1</string>
  <key>CFBundleExecutable</key><string>$2</string>
  <key>CFBundleIdentifier</key><string>$3</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>$version</string>
  <key>CFBundleVersion</key><string>$version</string>
  <key>LSMinimumSystemVersion</key><string>11.0</string>
$4
</dict>
</plist>
EOF
}

make_app "Broxer" broxer dev.dmbsk.broxer '  <key>LSUIElement</key><true/>
  <key>CFBundleURLTypes</key>
  <array><dict>
    <key>CFBundleURLName</key><string>Web URL</string>
    <key>CFBundleURLSchemes</key><array><string>http</string><string>https</string></array>
  </dict></array>'

make_app "Broxer Settings" broxer-settings dev.dmbsk.broxer-settings ''

echo "Built dist/Broxer.app and dist/Broxer Settings.app — move both to /Applications."
