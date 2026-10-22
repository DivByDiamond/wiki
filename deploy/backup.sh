#!/bin/sh
# Копия базы раз в неделю (cron/systemd timer) и перед обновлением.
set -eu
DB="${WIKI_DB:-/var/lib/oc2r-wiki/wiki.db}"
OUT="${1:-/var/backups/oc2r-wiki}"
mkdir -p "$OUT"
sqlite3 "$DB" ".backup '$OUT/wiki-$(date +%F).db'"
find "$OUT" -name 'wiki-*.db' -mtime +60 -delete
