#!/usr/bin/env bash
set -euo pipefail

# Package surrogate-macos into a .app bundle.
# Uses native GUI (cocoanut + tray-icon), no Tauri dependency.
#
# Usage:
#   ./scripts/package-macos.sh [debug|release] [--target <triple>] [--skip-build]
#
# Examples:
#   ./scripts/package-macos.sh                                      # debug, host target
#   ./scripts/package-macos.sh release                               # release, host target
#   ./scripts/package-macos.sh release --target aarch64-apple-darwin  # release, explicit target
#   ./scripts/package-macos.sh release --skip-build                  # skip cargo build

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
APP_NAME="Surrogate"
BUNDLE="${PROJECT_ROOT}/${APP_NAME}.app"
CRATE_DIR="${PROJECT_ROOT}/crates/surrogate-macos"

PROFILE="${1:-debug}"
shift || true

TARGET=""
SKIP_BUILD=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        --target)
            TARGET="${2:?--target requires a value}"
            shift 2
            ;;
        --skip-build)
            SKIP_BUILD=true
            shift
            ;;
        *)
            echo "Unknown option: $1" >&2
            exit 1
            ;;
    esac
done

if [ "$PROFILE" = "release" ]; then
    BUILD_FLAG="--release"
    TARGET_DIR="release"
else
    BUILD_FLAG=""
    TARGET_DIR="debug"
fi

if [ -n "$TARGET" ]; then
    BINARY="${PROJECT_ROOT}/target/${TARGET}/${TARGET_DIR}/surrogate-macos"
    BUILD_TARGET_FLAG="--target ${TARGET}"
else
    BINARY="${PROJECT_ROOT}/target/${TARGET_DIR}/surrogate-macos"
    BUILD_TARGET_FLAG=""
fi

if [ "$SKIP_BUILD" = false ]; then
    echo "==> Building surrogate-macos ($PROFILE${TARGET:+, target=$TARGET})..."
    # shellcheck disable=SC2086
    cargo build $BUILD_FLAG $BUILD_TARGET_FLAG -p surrogate-macos
fi

if [ ! -f "$BINARY" ]; then
    echo "ERROR: Binary not found at $BINARY"
    echo "       If built with --target, pass it here too."
    exit 1
fi

echo "==> Verifying binary is surrogate-macos (not surrogate-app)..."
BINARY_NAME=$(basename "$(readlink -f "$BINARY" 2>/dev/null || echo "$BINARY")")
if echo "$BINARY_NAME" | grep -q "surrogate-app"; then
    echo "ERROR: Binary appears to be surrogate-app, not surrogate-macos."
    echo "       Build with: cargo build -p surrogate-macos"
    exit 1
fi

echo "==> Assembling ${APP_NAME}.app..."
rm -rf "$BUNDLE"
mkdir -p "${BUNDLE}/Contents/MacOS"
mkdir -p "${BUNDLE}/Contents/Resources"

cp "$BINARY" "${BUNDLE}/Contents/MacOS/${APP_NAME}"

for icon in "${CRATE_DIR}"/icons/*.png; do
    [ -f "$icon" ] && cp "$icon" "${BUNDLE}/Contents/Resources/"
done

VERSION=$(grep '^version' "${PROJECT_ROOT}/Cargo.toml" | head -1 | sed 's/.*"\(.*\)".*/\1/')

cat > "${BUNDLE}/Contents/Info.plist" << PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key>
  <string>${APP_NAME}</string>
  <key>CFBundleDisplayName</key>
  <string>${APP_NAME}</string>
  <key>CFBundleIdentifier</key>
  <string>com.yorha-agents.surrogate</string>
  <key>CFBundleVersion</key>
  <string>${VERSION}</string>
  <key>CFBundleShortVersionString</key>
  <string>${VERSION}</string>
  <key>CFBundleExecutable</key>
  <string>${APP_NAME}</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>LSMinimumSystemVersion</key>
  <string>11.0</string>
  <key>LSApplicationCategoryType</key>
  <string>public.app-category.utilities</string>
  <key>NSHighResolutionCapable</key>
  <true/>
  <key>LSUIElement</key>
  <true/>
  <key>NSAppTransportSecurity</key>
  <dict>
    <key>NSAllowsArbitraryLoads</key>
    <true/>
    <key>NSAllowsLocalNetworking</key>
    <true/>
  </dict>
  <key>NSLocalNetworkUsageDescription</key>
  <string>Surrogate needs local network access to operate as a proxy server.</string>
</dict>
</plist>
PLIST

if [ -f "${CRATE_DIR}/Surrogate.entitlements" ]; then
    cp "${CRATE_DIR}/Surrogate.entitlements" "${BUNDLE}/Contents/Resources/"
fi

echo "==> Done: ${BUNDLE}"
echo "    Run: open ${BUNDLE}"
