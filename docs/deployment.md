# Публикация на VPS

Действующая установка: `root@185.176.94.38`, сайт `https://wiki.loki-code.dev`.
Приложение слушает `127.0.0.1:8091`; nginx принимает HTTPS за Cloudflare.
База: `/var/lib/oc2r-wiki/wiki.db`; бинарник: `/usr/local/bin/oc2r-wiki`.
Сервис работает от пользователя `wiki`. Настройки — `/etc/oc2r-wiki.env` (root, 600).

## Сборка

На машине сборки glibc новее серверной, поэтому для VPS используется статическая musl-сборка:

```bash
(cd web && bun install --frozen-lockfile && bun run build && bun run check && bun test)
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo build --release --target x86_64-unknown-linux-musl -p oc2r-wiki
```

Нужны установленная цель `x86_64-unknown-linux-musl` и `musl-gcc`.
Готовый файл: `target/x86_64-unknown-linux-musl/release/oc2r-wiki`.
Сверяйте SHA-256 после передачи на сервер.

## Порядок обновления

1. Сохранить кандидат в `/opt/oc2r-wiki/releases/<commit>/oc2r-wiki`.
2. Сохранить предыдущий бинарник, настройки и консистентную SQLite-копию
   в отдельный каталог `backup-<timestamp>` с правами 700 внутри каталога релиза.
   Для SQLite использовать `.backup` или SQLite Backup API; простое копирование
   активного файла в WAL-режиме не обеспечивает консистентную копию.
3. Остановить только `oc2r-wiki`. При обновлении содержимого выполнить кандидатом
   `--sync-content` с окружением сервиса от пользователя `wiki`.
4. Установить файл через временный соседний путь и атомарное переименование в
   `/usr/local/bin/oc2r-wiki`, затем запустить сервис.
5. Проверить `/health`, статью и ассеты по внутреннему адресу и внешнему HTTPS,
   `systemctl is-active oc2r-wiki nginx` и целостность базы.
6. При сбое остановить сервис, вернуть сохранённый бинарник и при необходимости
   восстановить базу через SQLite, затем запустить прежнюю версию.

При обновлении только оформления повторный импорт страниц не нужен.
Сессии хранятся в памяти: после перезапуска пользователи входят заново.

## GitHub

В приложении `oc2r-wiki` зарегистрирован callback
`https://wiki.loki-code.dev/auth/github/callback`. Он должен совпадать с
`PUBLIC_URL` плюс `/auth/github/callback`. Неверный список callback исправляется
в настройках GitHub App; повторное открытие «Войти» запускает новый OAuth flow.

Client ID, client secret и `ADMIN_GITHUB_IDS` уже настроены на VPS. Не добавляйте
секреты в Git, аргументы команд или вывод диагностик.

## Резервные копии

`oc2r-wiki-backup.timer` включён; его сервис выполняет SQLite backup в
`/var/lib/oc2r-wiki/backups` и сохраняет копии за 60 дней. Проверка вручную:

```bash
systemctl start oc2r-wiki-backup.service
systemctl show oc2r-wiki-backup.service -p Result -p ExecMainStatus
systemctl list-timers oc2r-wiki-backup.timer
```
