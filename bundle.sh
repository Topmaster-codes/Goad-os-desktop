#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
set -euo pipefail

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
OUTPUT="${1:-$ROOT/goad-os-amane.tar.gz}"
OUTPUT_DIR="$(dirname -- "$OUTPUT")"
mkdir -p -- "$OUTPUT_DIR"
OUTPUT_DIR="$(cd -- "$OUTPUT_DIR" && pwd)"
OUTPUT="$OUTPUT_DIR/$(basename -- "$OUTPUT")"
TEMP_ARCHIVE="$(mktemp "${TMPDIR:-/tmp}/goad-os-amane.XXXXXX")"
trap 'rm -f -- "$TEMP_ARCHIVE"' EXIT

if ! git -C "$ROOT" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    printf 'Run bundle.sh from a Git checkout; refusing to archive an unfiltered directory.\n' >&2
    exit 1
fi

# Include tracked project files only. Exclude generated build output and
# common local credential files even if they were accidentally tracked.
git -C "$ROOT" ls-files -z -- . ':!:target' ':!:target/**' |
    tar --create --gzip --file="$TEMP_ARCHIVE" \
        --directory="$ROOT" \
        --null --files-from=- \
        --exclude='./.env' \
        --exclude='./.env.*' \
        --exclude='./*.pem' \
        --exclude='./*.key' \
        --exclude='./*.p12' \
        --exclude='./*.pfx' \
        --exclude='./.aws' \
        --exclude='./.codex' \
        --exclude='./.agents'

chmod 644 "$TEMP_ARCHIVE"
mv -f -- "$TEMP_ARCHIVE" "$OUTPUT"
printf 'Created %s\n' "$OUTPUT"
