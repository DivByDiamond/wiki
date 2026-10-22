#!/bin/sh
# Копия базы раз в неделю (oc2r-wiki-backup.timer) и перед обновлением.
# Installed as /usr/local/bin/oc2r-wiki-backup. Нет базы - выходим с кодом 0.
set -eu
DB="${WIKI_DB:-/var/lib/oc2r-wiki/wiki.db}"
OUT="${1:-/var/backups/oc2r-wiki}"
if [ ! -f "$DB" ]; then
    echo "oc2r-wiki-backup: no database at $DB, nothing to do"
    exit 0
fi
mkdir -p "$OUT"
sqlite3 "$DB" ".backup '$OUT/wiki-$(date +%F).db'"
find "$OUT" -name 'wiki-*.db' -mtime +60 -delete
