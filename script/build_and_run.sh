#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."
MODE="${1:-run}"
case "$MODE" in
  run|--verify) ;;
  *) echo "usage: $0 [run|--verify]" >&2; exit 2 ;;
esac

pkill -x recorder >/dev/null 2>&1 || true
./scripts/bundle.sh
APP="${DEST:-$HOME/Applications}/Recorder.app"
/usr/bin/open -n "$APP"
if [ "$MODE" = --verify ]; then
  sleep 2
  pgrep -x recorder
fi
