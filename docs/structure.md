# Структура репозитория и правила кода

## Правила

1. **Не больше 4 файлов кода в папке.** Папки считаются отдельно, подпапки не входят в счёт.
2. **Не больше 250 строк в файле кода, для `.css` до 500.** Превысили — делим по ответственности.
   Оба лимита только для кода (`.rs`, `.ts`, `.css`, шаблоны, `.sql`). Документы, конфиги,
   `LICENSE`, `Cargo.lock` и `todo.md` лимитам не подчиняются.
3. **KISS:** самое простое решение, которое закрывает требование. Никакого запаса «на потом».
4. **DRY:** один конвейер рендера, один `AppError`, одна политика прав, одна функция записи ревизии.
5. **SOLID:**
   - один модуль — одна причина измениться;
   - `domain` определяет порты (trait), `store` их реализует, `http` получает их через `AppState`;
   - новые виды правок или ролей добавляются новыми модулями, а не правкой старых `match`;
   - трейты узкие: `PageRepo`, `ProposalRepo`, `AccessRepo`, а не один общий `Store`.
6. **Лицензия:** GPL-3.0-or-later. `LICENSE` в корне, в `Cargo.toml` поле `license`.

## Корень

```
Cargo.toml        workspace
LICENSE           GPL-3.0
README.md
todo.md           план работ
docs/             design.md  workflows.md  reader.md  structure.md
server/           Rust-крейт
web/              TypeScript, без библиотек
deploy/           Caddyfile  oc2r-wiki.service  backup.sh
```

## server/

```
server/Cargo.toml
server/src/
  main.rs  lib.rs  config.rs  error.rs   (lib.rs: список модулей и router, main.rs только запуск)
  domain/        mod.rs  page.rs  revision.rs  proposal.rs
  domain/ports/  mod.rs  pages.rs  proposals.rs  access.rs
  diff/          mod.rs  lines.rs
  render/        mod.rs  markdown.rs  extensions.rs  sanitize.rs
  auth/          mod.rs  github.rs  policy.rs
  auth/tokens/   mod.rs  session.rs  api_key.rs
  store/         mod.rs  db.rs
  store/content/ mod.rs  pages.rs  revisions.rs  search.rs
  store/access/  mod.rs  users.rs  keys.rs  proposals.rs
  store/sql/     0001_content.sql  0002_access.sql  0003_search.sql
  http/          mod.rs  state.rs  middleware.rs  rate_limit.rs
  http/reader/   mod.rs  articles.rs  history.rs  search.rs
  http/user/     mod.rs  login.rs  propose.rs
  http/admin/    mod.rs  pages.rs  proposals.rs  keys.rs
  http/api/      mod.rs  pages.rs  export.rs
server/templates/   reader/  admin/  layout/   (в каждой не больше 4 файлов)
server/tests/       e2e.rs  support/mod.rs
server/tests/public/  main.rs  api.rs  reader.rs
server/tests/staff/   main.rs  proposals.rs  admin.rs
```

Назначение слоёв:
- `domain`: типы и правила без ввода-вывода. Например «откат — это новая ревизия», «конфликт, если
  текущая ревизия новее `base_rev_id`».
- `diff`: чистая функция сравнения двух текстов, ничего не знает про HTML.
- `render`: Markdown в безопасный HTML. Единственный путь рендера в проекте.
- `auth`: OAuth, сессии, ключи, проверка прав. Хранилище доступно только через порты.
- `store`: SQLite. Только запросы и отображение строк в типы `domain`.
- `http`: маршруты тонкие. Разобрать запрос, вызвать политику и порты, собрать ответ.

Если нужно больше файлов, чем даёт правило, создаётся новая подпапка по смыслу,
а не 5-й файл в той же.

## web/

```
web/package.json  tsconfig.json  build.ts
web/src/
  main.ts
  settings/  store.ts  theme.ts  panel.ts
  search/    index.ts  ui.ts
  editor/    editor.ts  preview.ts  toolbar.ts  diffview.ts
  shared/    dom.ts  http.ts  keys.ts
web/styles/  tokens.css  layout.css  components.css  pixel.css
```

- Инструмент один: Bun (`bun install`, `bun test`, `Bun.build`). Next.js и другие фреймворки не используем. Результат в `web/dist`, затем встраивается в бинарник сервера.
- `tokens.css` — переменные (цвета, акцент, `--pixel`). `pixel.css` включается переменной, а не
  дублирует стили.
- Скрипт темы в `<head>` — отдельная маленькая строка в шаблоне, а не часть бандла.
- Зависимости рантайма во фронтенде: ноль.

## Зависимости Rust (проверяются при планировании)

axum, tokio, rusqlite (или sqlx), pulldown-cmark, ammonia (очистка HTML), sha2, rand,
reqwest (GitHub OAuth), serde, tracing, rust-embed. Лишнего не добавляем.
