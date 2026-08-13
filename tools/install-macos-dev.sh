#!/bin/sh

set -eu

if [ "$(uname -s)" != "Darwin" ]; then
  printf '%s\n' "install:mac is only available on macOS." >&2
  exit 1
fi

REPO_ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
BUILD_APP="$REPO_ROOT/target/debug/bundle/macos/Tro.app"
INSTALL_APP="/Applications/Tro.app"
TRO_EXECUTABLE="$INSTALL_APP/Contents/MacOS/desktop"
TRO_DEV_LOG="${TMPDIR:-/tmp}/tro-dev.log"
TRO_DEV_JOB="vn.tro.desktop.doppler-dev"
TRO_DOPPLER_PROJECT="tro"
TRO_DOPPLER_CONFIG="dev"

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
/bin/launchctl remove "$TRO_DEV_JOB" 2>/dev/null || true
/usr/bin/pkill -f '^/Applications/Tro\.app/Contents/MacOS/desktop$' 2>/dev/null || true
attempt=0
while /usr/bin/pgrep -f '^/Applications/Tro\.app/Contents/MacOS/desktop$' >/dev/null; do
  attempt=$((attempt + 1))
  if [ "$attempt" -ge 50 ]; then
    printf '%s\n' "Tro did not exit in time." >&2
    exit 1
  fi
  sleep 0.1
done
/usr/bin/ditto "$BUILD_APP" "$INSTALL_APP"

if ! command -v doppler >/dev/null 2>&1; then
  printf '%s\n' "Doppler CLI is required to launch the development app." >&2
  exit 1
fi
DOPPLER_EXECUTABLE=$(command -v doppler)
if ! doppler run --project "$TRO_DOPPLER_PROJECT" --config "$TRO_DOPPLER_CONFIG" -- \
  sh -c 'test -n "${OPENROUTER_API_KEY:-}"'; then
  printf '%s\n' "OPENROUTER_API_KEY is missing from the active Doppler config." >&2
  exit 1
fi

# Keep development credentials out of the app bundle and local plaintext files.
# Doppler supplies them only to the running Tro process.
/bin/launchctl submit \
  -l "$TRO_DEV_JOB" \
  -o "$TRO_DEV_LOG" \
  -e "$TRO_DEV_LOG" \
  -- "$DOPPLER_EXECUTABLE" run \
  --project "$TRO_DOPPLER_PROJECT" \
  --config "$TRO_DOPPLER_CONFIG" \
  -- "$TRO_EXECUTABLE"
attempt=0
while ! /usr/bin/pgrep -f '^/Applications/Tro\.app/Contents/MacOS/desktop$' >/dev/null; do
  attempt=$((attempt + 1))
  if [ "$attempt" -ge 50 ]; then
    /bin/launchctl remove "$TRO_DEV_JOB" 2>/dev/null || true
    printf '%s\n' "Tro did not start in time. See $TRO_DEV_LOG." >&2
    exit 1
  fi
  sleep 0.1
done

printf '%s\n' "Installed and started the Doppler-injected development build at $INSTALL_APP"
