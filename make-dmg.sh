#!/bin/bash
# Builds ClipFlow.app and packs it into ClipFlow.dmg.
# Run from anywhere: ./make-dmg.sh
set -euo pipefail
cd "$(dirname "$0")"

APP="build/ClipFlow.app"
DMG="ClipFlow.dmg"

cargo build --release

rm -rf build
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp target/release/clipflow "$APP/Contents/MacOS/ClipFlow"
cp assets/AppIcon.icns "$APP/Contents/Resources/AppIcon.icns"

cat > "$APP/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleDisplayName</key>
    <string>ClipFlow</string>
    <key>CFBundleExecutable</key>
    <string>ClipFlow</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>CFBundleIdentifier</key>
    <string>app.clipflow.simple</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>ClipFlow</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>LSApplicationCategoryType</key>
    <string>public.app-category.productivity</string>
    <key>LSMinimumSystemVersion</key>
    <string>11.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSPrincipalClass</key>
    <string>NSApplication</string>
</dict>
</plist>
PLIST

# Ad-hoc signature: the arm64 binary needs one, and it keeps Gatekeeper calm.
codesign --force --deep --sign - "$APP" >/dev/null 2>&1 || true

mkdir -p build/dmg
cp -R "$APP" build/dmg/
ln -s /Applications build/dmg/Applications

rm -f "$DMG"
hdiutil create -quiet -volname "ClipFlow" -srcfolder build/dmg -ov -format UDZO "$DMG"

echo "Wrote $(pwd)/$DMG"
