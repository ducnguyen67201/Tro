#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
fixture_root="$repo_root/tests/fixtures/computer-use"
profile_dir=$(mktemp -d "${TMPDIR:-/tmp}/tro-browser-profile.XXXXXX")
port=${TRO_FIXTURE_PORT:-8765}

cleanup() {
  if [ -n "${server_pid:-}" ]; then
    kill "$server_pid" 2>/dev/null || true
    wait "$server_pid" 2>/dev/null || true
  fi
  rm -rf "$profile_dir"
}
trap cleanup EXIT INT TERM

python3 -m http.server "$port" --bind 127.0.0.1 --directory "$fixture_root" &
server_pid=$!
fixture_url="http://127.0.0.1:$port/course-browser.html"
launch_delay=${TRO_FIXTURE_LAUNCH_DELAY_SECONDS:-0}
case "$launch_delay" in
  ''|*[!0-9.]*)
    echo "TRO_FIXTURE_LAUNCH_DELAY_SECONDS must be a non-negative number." >&2
    exit 1
    ;;
esac
sleep "$launch_delay"

case "$(uname -s)" in
  Darwin)
    chrome="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
    if [ ! -x "$chrome" ]; then
      echo "Google Chrome is required for this supervised fixture." >&2
      exit 1
    fi
    "$chrome" --user-data-dir="$profile_dir" --no-first-run "$fixture_url"
    ;;
  *)
    echo "Open $fixture_url in an isolated temporary browser profile." >&2
    echo "The Windows signed-device run remains supervised." >&2
    wait "$server_pid"
    ;;
esac
