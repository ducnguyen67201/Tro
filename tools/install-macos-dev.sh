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
TRO_API_EXECUTABLE="$REPO_ROOT/target/debug/api"
TRO_DESKTOP_LOG="${TMPDIR:-/tmp}/tro-desktop-dev.log"
TRO_API_LOG="${TMPDIR:-/tmp}/tro-api-dev.log"
TRO_DESKTOP_JOB="vn.tro.desktop.doppler-dev"
TRO_API_JOB="vn.tro.api.doppler-dev"
TRO_DESKTOP_PLIST="${TMPDIR:-/tmp}/$TRO_DESKTOP_JOB.plist"
TRO_DOPPLER_PROJECT="tro"
TRO_BACKEND_CONFIG="dev"
TRO_API_BIND_ADDR="127.0.0.1:18080"
TRO_API_URL="http://$TRO_API_BIND_ADDR"
TRO_HEALTH_URL="$TRO_API_URL/healthz"

cd "$REPO_ROOT"
cargo build -p api
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
/bin/launchctl remove "$TRO_DESKTOP_JOB" 2>/dev/null || true
/bin/launchctl remove "$TRO_API_JOB" 2>/dev/null || true
/usr/bin/pkill -f '^/Applications/Tro\.app/Contents/MacOS/desktop$' 2>/dev/null || true
/usr/bin/pkill -f "$TRO_API_EXECUTABLE" 2>/dev/null || true
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
if ! doppler run --project "$TRO_DOPPLER_PROJECT" --config "$TRO_BACKEND_CONFIG" -- \
  sh -c 'test -n "${OPENROUTER_API_KEY:-}" && test -n "${TRO_DEVICE_TOKEN:-}"'; then
  printf '%s\n' "Backend credentials are missing from Doppler." >&2
  exit 1
fi
# The backend receives provider credentials. The desktop receives no provider
# credential and establishes its own revocable session through onboarding.
/bin/launchctl submit \
  -l "$TRO_API_JOB" \
  -o "$TRO_API_LOG" \
  -e "$TRO_API_LOG" \
  -- /usr/bin/env -i \
  HOME="$HOME" \
  PATH=/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin \
  "$DOPPLER_EXECUTABLE" run \
  --project "$TRO_DOPPLER_PROJECT" \
  --config "$TRO_BACKEND_CONFIG" \
  -- /usr/bin/env \
  AGENT_ENABLED=true \
  BIND_ADDR="$TRO_API_BIND_ADDR" \
  REALTIME_ENABLED=false \
  TRO_DEVELOPMENT_INVITE_CODE=TRO-LOCAL \
  "$TRO_API_EXECUTABLE"
attempt=0
while ! /usr/bin/curl --fail --silent --max-time 1 "$TRO_HEALTH_URL" >/dev/null; do
  attempt=$((attempt + 1))
  if [ "$attempt" -ge 100 ]; then
    /bin/launchctl remove "$TRO_API_JOB" 2>/dev/null || true
    printf '%s\n' "Tro API did not start in time. See $TRO_API_LOG." >&2
    exit 1
  fi
  sleep 0.1
done

# A submitted launchctl job infers KeepAlive=true and makes Quit relaunch the
# app. Use an explicit one-shot LaunchAgent so Tro starts once and stays off.
/usr/bin/plutil -create xml1 "$TRO_DESKTOP_PLIST"
/usr/bin/plutil -insert Label -string "$TRO_DESKTOP_JOB" "$TRO_DESKTOP_PLIST"
/usr/bin/plutil -insert ProgramArguments -array "$TRO_DESKTOP_PLIST"
/usr/bin/plutil -insert ProgramArguments.0 -string /usr/bin/env "$TRO_DESKTOP_PLIST"
/usr/bin/plutil -insert ProgramArguments.1 -string -i "$TRO_DESKTOP_PLIST"
/usr/bin/plutil -insert ProgramArguments.2 -string "HOME=$HOME" "$TRO_DESKTOP_PLIST"
/usr/bin/plutil -insert ProgramArguments.3 -string PATH=/usr/bin:/bin:/usr/sbin:/sbin "$TRO_DESKTOP_PLIST"
/usr/bin/plutil -insert ProgramArguments.4 -string TRO_DEV_MANAGED_BACKEND=1 "$TRO_DESKTOP_PLIST"
/usr/bin/plutil -insert ProgramArguments.5 -string "TRO_API_BASE_URL=$TRO_API_URL" "$TRO_DESKTOP_PLIST"
/usr/bin/plutil -insert ProgramArguments.6 -string "$TRO_EXECUTABLE" "$TRO_DESKTOP_PLIST"
/usr/bin/plutil -insert RunAtLoad -bool true "$TRO_DESKTOP_PLIST"
/usr/bin/plutil -insert KeepAlive -bool false "$TRO_DESKTOP_PLIST"
/usr/bin/plutil -insert ProcessType -string Interactive "$TRO_DESKTOP_PLIST"
/usr/bin/plutil -insert StandardOutPath -string "$TRO_DESKTOP_LOG" "$TRO_DESKTOP_PLIST"
/usr/bin/plutil -insert StandardErrorPath -string "$TRO_DESKTOP_LOG" "$TRO_DESKTOP_PLIST"
/bin/launchctl bootstrap "gui/$(id -u)" "$TRO_DESKTOP_PLIST"
attempt=0
while ! /usr/bin/pgrep -f '^/Applications/Tro\.app/Contents/MacOS/desktop$' >/dev/null; do
  attempt=$((attempt + 1))
  if [ "$attempt" -ge 50 ]; then
    /bin/launchctl remove "$TRO_DESKTOP_JOB" 2>/dev/null || true
    /bin/launchctl remove "$TRO_API_JOB" 2>/dev/null || true
    /usr/bin/pkill -f '^/Applications/Tro\.app/Contents/MacOS/desktop$' 2>/dev/null || true
    printf '%s\n' "Tro did not start in time. See $TRO_DESKTOP_LOG." >&2
    exit 1
  fi
  sleep 0.1
done

printf '%s\n' "Installed Tro and started the stoppable desktop with its isolated backend."
