#!/bin/sh

set -eu

if [ "$(uname -s)" != "Darwin" ]; then
  printf '%s\n' "install:mac is only available on macOS." >&2
  exit 1
fi

REPO_ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
BUILD_APP="$REPO_ROOT/target/debug/bundle/macos/Tro.app"
INSTALL_APP="/Applications/Tro.app"

cd "$REPO_ROOT"
pnpm --filter @tro/desktop exec tauri build --debug --bundles app

# Unsigned debug builds default to a changing CDHash requirement. macOS binds
# TCC permissions to that requirement, so Accessibility and Input Monitoring
# silently stop matching after every rebuild. This explicit development-only
# requirement keeps the local identity stable until release signing is set up.
printf '%s\n' 'designated => identifier "vn.tro.desktop"' |
  /usr/bin/codesign \
    --force \
    --deep \
    --sign - \
    --identifier vn.tro.desktop \
    -r - \
    "$BUILD_APP"

/usr/bin/codesign --verify --deep --strict --verbose=2 "$BUILD_APP"
/usr/bin/pkill -f '^/Applications/Tro\.app/Contents/MacOS/desktop$' 2>/dev/null || true
/usr/bin/ditto "$BUILD_APP" "$INSTALL_APP"
/usr/bin/open -a "$INSTALL_APP"

printf '%s\n' "Installed the stable development build at $INSTALL_APP"
