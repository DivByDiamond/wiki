<div align="center">

# OC2R Wiki

Wiki for the [OC2R] Minecraft mod, with page history, reader edit suggestions and a key-protected API.
Rust backend, TypeScript frontend, one binary, one SQLite file.

![Rust](https://img.shields.io/badge/Rust-2024-orange?style=flat-square&logo=rust&logoColor=white)
![TypeScript](https://img.shields.io/badge/TypeScript-Bun-3178c6?style=flat-square&logo=typescript&logoColor=white)
![SQLite](https://img.shields.io/badge/SQLite-FTS5-003b57?style=flat-square&logo=sqlite&logoColor=white)
![License](https://img.shields.io/badge/license-GPLv3-blue?style=flat-square&logo=gnu&logoColor=white)
![Status](https://img.shields.io/badge/status-in%20development-yellow?style=flat-square)

[English](#english) | [Русский](#русский)

</div>

---

> **Status:** design and implementation plan are written, code is not started yet. See [`docs/`](docs) and [`todo.md`](todo.md).

---

## English

### Overview

OC2R Wiki is the documentation site for [OC2R] at `wiki.loki-code.dev`, aimed at players and script authors. Pages live in a database, every edit is a revision, and any old version can be opened, compared and restored.

Admins and API keys edit pages directly. Everyone else signs in with GitHub and sends a suggestion, which an admin accepts or rejects.

### Features

| Feature | Description |
|---------|-------------|
| Page history | Every edit is an immutable revision with author and comment. Open any version, diff two versions, roll back |
| Edit suggestions | Readers sign in with GitHub and propose changes, admins review a diff and accept or reject |
| API by key | `PUT` pages from CI or scripts with a scoped key (`read`, `write`, `export`), keys are stored hashed |
| Admin panel | Markdown editor with live preview, suggestion queue, history and rollback, key management |
| Server-side rendering | Markdown is rendered and sanitized on the server, pages work without JavaScript |
| Search | Full-text search (SQLite FTS5) |
| Languages | English, Russian, Chinese, with fallback to English |
| Reader settings | Light or dark theme, accent color, font size, optional pixel borders, stored in the browser |

### Tech stack

- **Backend:** Rust (edition 2024), axum, SQLite (rusqlite), pulldown-cmark, ammonia
- **Frontend:** TypeScript without runtime libraries, built with Bun
- **Deploy:** one binary and one database file behind Caddy

### Build

Planned commands, they will work once the first tasks of [`todo.md`](todo.md) are done:

```bash
cd web && bun install --frozen-lockfile && bun run build
cd .. && cargo build --release -p oc2r-wiki
```

### Configuration

| Variable | Description |
|----------|-------------|
| `WIKI_BIND` | Listen address, default `127.0.0.1:8080` |
| `WIKI_DB` | Path to the SQLite file, default `wiki.db` |
| `GITHUB_CLIENT_ID` / `GITHUB_CLIENT_SECRET` | GitHub login credentials |
| `PUBLIC_URL` | Public site URL, used for the login callback |
| `ADMIN_LOGINS` | Comma-separated GitHub logins that become admins |

### API example

```bash
curl -X PUT -H "Authorization: Bearer $KEY" --data-binary @page.md \
  "https://wiki.loki-code.dev/api/pages/start?lang=en&summary=sync"
```

### Documentation

- [`docs/design.md`](docs/design.md) - decisions, architecture, data model, deploy
- [`docs/workflows.md`](docs/workflows.md) - roles, history, suggestions, admin panel, API
- [`docs/reader.md`](docs/reader.md) - sections, style, settings, errors, tests
- [`docs/structure.md`](docs/structure.md) - repository layout and code rules
- [`todo.md`](todo.md) - implementation plan

### License

GPL-3.0-or-later, see [LICENSE](LICENSE).

---

## Русский

### Обзор

OC2R Wiki - сайт документации мода [OC2R] на `wiki.loki-code.dev` для игроков и авторов скриптов. Страницы хранятся в базе, каждая правка - это ревизия, а любую старую версию можно открыть, сравнить и восстановить.

Админы и API-ключи правят страницы напрямую. Остальные входят через GitHub и отправляют предложения, которые админ принимает или отклоняет.

### Возможности

| Возможность | Описание |
|-------------|----------|
| История страниц | Каждая правка - неизменяемая ревизия с автором и комментарием. Можно открыть любую версию, сравнить две версии, откатить |
| Предложения правок | Читатели входят через GitHub и предлагают изменения, админ смотрит diff и принимает или отклоняет |
| API по ключу | `PUT` страниц из CI или скриптов ключом с правами (`read`, `write`, `export`), ключи хранятся хешами |
| Админка | Markdown-редактор с живым превью, очередь предложений, история и откат, управление ключами |
| Рендер на сервере | Markdown рендерится и очищается на сервере, страницы работают без JavaScript |
| Поиск | Полнотекстовый поиск (SQLite FTS5) |
| Языки | Английский, русский, китайский, с откатом на английский |
| Настройки читателя | Светлая или тёмная тема, акцентный цвет, размер шрифта, пиксель-рамки по желанию, хранятся в браузере |

### Технологии

- **Бэкенд:** Rust (edition 2024), axum, SQLite (rusqlite), pulldown-cmark, ammonia
- **Фронтенд:** TypeScript без рантайм-библиотек, сборка через Bun
- **Деплой:** один бинарник и один файл базы за Caddy

### Сборка

Планируемые команды, заработают после первых задач из [`todo.md`](todo.md):

```bash
cd web && bun install --frozen-lockfile && bun run build
cd .. && cargo build --release -p oc2r-wiki
```

### Настройка

| Переменная | Описание |
|------------|----------|
| `WIKI_BIND` | Адрес для прослушивания, по умолчанию `127.0.0.1:8080` |
| `WIKI_DB` | Путь к файлу SQLite, по умолчанию `wiki.db` |
| `GITHUB_CLIENT_ID` / `GITHUB_CLIENT_SECRET` | Данные приложения GitHub для входа |
| `PUBLIC_URL` | Публичный адрес сайта, нужен для callback входа |
| `ADMIN_LOGINS` | Логины GitHub через запятую, которые становятся админами |

### Пример API

```bash
curl -X PUT -H "Authorization: Bearer $KEY" --data-binary @page.md \
  "https://wiki.loki-code.dev/api/pages/start?lang=en&summary=sync"
```

### Документация

- [`docs/design.md`](docs/design.md) - решения, архитектура, данные, деплой
- [`docs/workflows.md`](docs/workflows.md) - роли, история, предложения, админка, API
- [`docs/reader.md`](docs/reader.md) - разделы, стиль, настройки, ошибки, тесты
- [`docs/structure.md`](docs/structure.md) - структура репозитория и правила кода
- [`todo.md`](todo.md) - план реализации

### Лицензия

GPL-3.0-or-later, см. [LICENSE](LICENSE).

[OC2R]: https://github.com/DivByDiamond/OC2R
