# OC2R Wiki: статус и план

Исторический пошаговый план реализации (задачи 0–14 с кодом) удалён: всё реализовано, код лежит в репозитории, история в git.
Спецификация: `docs/design.md`, `docs/workflows.md`, `docs/reader.md`, `docs/structure.md`, `docs/ux.md`.
Следующий этап (QoL и дизайн): [docs/roadmap.md](docs/roadmap.md).

## Current completion audit (2026-10-23)

The historical step checkboxes below include test-first and commit instructions;
they are preserved as the original plan rather than claimed execution records.
Current implementation status is tracked here:

- [x] Tasks 0–9: workspace/health, errors, domain/ports, bounded diff, sanitized Markdown, policy/keys, SQLite migrations, revisions/FTS, sessions and API/export.
- [x] Task 10: server-rendered articles, English fallback, history/diff/old revisions, navigation, table of contents, previous/next links and reader 404.
- [x] Task 11: GitHub OAuth flow and edit proposals, including quotas and permission checks; integration tests use a fake provider.
- [x] Task 12 final admin usability (localized UI, searchable/filterable table, persistent navigation and editor preview): grouped dashboard, discoverable page creation and validated section/order controls; navigation-only edits are recorded and protected from imports.
- [x] Task 13: built assets, persisted reader settings, localized controls, search, editor/merge UI; editor preview request race fixed with a regression test.
- [x] Task 14 local validation: 245 Rust tests, 8 frontend tests, 3 converter tests, TypeScript, clippy with warnings denied, release build, code structure and browser acceptance passed.
- [x] Content migration beyond the original plan: 134 pages from a pinned OC2R revision, images, converted links and safe transactional synchronization.
- [x] Reader search results highlight matching text with escaped snippets.
- [x] Deployment-environment verification: deployed on VPS behind nginx/Cloudflare; HTTPS checks pass, real GitHub sign-in and admin role confirmed, backup unit succeeded and weekly timer is enabled.

Content maintenance: [docs/content.md](docs/content.md).

## Global Constraints

- Лицензия GPL-3.0-or-later (`license = "GPL-3.0-or-later"` в каждом `Cargo.toml`).
- Rust `edition = "2024"`.
- Не больше 4 файлов кода в папке и не больше 250 строк в файле кода (`.rs`, `.ts`, шаблоны, `.sql`); для `.css` лимит строк 500. Документы и конфиги лимитам не подчиняются. Нужен файл сверх лимита — создаётся подпапка по смыслу.
- KISS, DRY, SOLID: один конвейер `render::to_html`, один `AppError`, одна политика `auth::policy::can`, зависимости только внутрь (`domain` не знает про `store` и `http`), узкие порты `PageRepo`, `ProposalRepo`, `AccessRepo`.
- Ревизии неизменяемы. Откат создаёт новую ревизию. Удаление страницы ставит `deleted_at`.
- Slug: сегменты `[a-z0-9_-]+`, разделитель `/`, не длиннее 100 символов. Языки: `en`, `ru`, `zh`.
- API-ключ: 32 случайных байта в hex с префиксом `ocw_`, в базе только SHA-256, показывается один раз.
- Права: `Actor::Anonymous`, `Actor::User { admin, blocked }`, `Actor::Key { read, write, export }`. Проверки только через `policy::can`.
- Ошибки API: `{"error": "код", "message": "текст"}`; коды 401, 403, 404, 422, 429 (с `Retry-After`).
- Фронтенд без рантайм-зависимостей и без фреймворков (не Next.js). Bun нужен только как инструмент: `bun test`, `Bun.build`, `bun install`.
- Хранилище синхронное (`Mutex<Connection>`), вызывается прямо из обработчиков: запросы к локальной SQLite занимают микросекунды.
- Коммиты: одна строка, без списков и без приписок об авторстве ИИ.

## Отложено

- Загрузка картинок и страница «Мои предложения» перенесены в [docs/roadmap.md](docs/roadmap.md) (QoL 4, 5).
