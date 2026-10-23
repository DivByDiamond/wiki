# OC2R Wiki Implementation Plan

> Historical implementation plan; unchecked boxes below are not a current status report.
> The implementation is present. Bundled content, reader completion and verification are documented in [docs/content.md](docs/content.md) and [docs/verification.md](docs/verification.md).

## Current completion audit (2026-10-23)

The historical step checkboxes below include test-first and commit instructions;
they are preserved as the original plan rather than claimed execution records.
Current implementation status is tracked here:

- [x] Tasks 0–9: workspace/health, errors, domain/ports, bounded diff, sanitized Markdown, policy/keys, SQLite migrations, revisions/FTS, sessions and API/export.
- [x] Task 10: server-rendered articles, English fallback, history/diff/old revisions, navigation, table of contents, previous/next links and reader 404.
- [x] Task 11: GitHub OAuth flow and edit proposals, including quotas and permission checks; integration tests use a fake provider.
- [x] Task 12 final admin usability: grouped dashboard, discoverable page creation and validated section/order controls; navigation-only edits are recorded and protected from imports.
- [x] Task 13: built assets, persisted reader settings, localized controls, search, editor/merge UI; editor preview request race fixed with a regression test.
- [x] Task 14 local validation: 235 Rust tests, 8 frontend tests, 3 converter tests, TypeScript, clippy with warnings denied, release build, code structure and browser acceptance passed.
- [x] Content migration beyond the original plan: 134 pages from a pinned OC2R revision, images, converted links and safe transactional synchronization.
- [x] Reader search results highlight matching text with escaped snippets.
- [ ] Deployment-environment verification: actual GitHub callback, production nginx/Cloudflare and backup timer, with configured credentials/server.

Media upload UI and a reader's own proposal list remain explicitly deferred in
“Вне этого плана” below. See [docs/content.md](docs/content.md) for content maintenance
and [docs/verification.md](docs/verification.md) for checks and verification limits.


> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Вики для мода OC2R на `wiki.loki-code.dev`: история ревизий, очередь предложений правок, админка и запись по API-ключу.

**Architecture:** Один Rust-бинарник (axum + SQLite). `domain` задаёт типы, правила и порты (trait), `store` реализует порты, `http` тонкий. Markdown рендерится на сервере одним конвейером. Фронтенд на TypeScript без рантайм-библиотек улучшает страницы (настройки, поиск, редактор).

**Tech Stack:** Rust edition 2024, axum, tokio, rusqlite (bundled, FTS5), pulldown-cmark, ammonia, sha2, rand, reqwest, serde, tracing, askama, rust-embed; TypeScript, Bun (сборка, тесты, менеджер пакетов; без Next.js и без фреймворков); nginx behind Cloudflare.

Спецификация: `docs/design.md`, `docs/workflows.md`, `docs/reader.md`, `docs/structure.md`.

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

## Структура файлов

См. `docs/structure.md`. Задачи ниже создают файлы ровно по ней.

---

### Task 0: Каркас workspace и health

**Files:**
- Create: `Cargo.toml`, `.gitignore`, `server/Cargo.toml`, `server/src/main.rs`, `server/src/lib.rs`, `server/src/config.rs`

**Interfaces:**
- Produces: `oc2r_wiki::router() -> axum::Router` (в `lib.rs`; крейт библиотечный, чтобы интеграционные тесты в `server/tests/` могли собрать приложение; `main.rs` только запускает сервер), `config::Config { bind: String, db_path: String }` с `Config::from_env()`.

- [ ] **Step 1: Создать manifest-файлы**

`Cargo.toml`:
```toml
[workspace]
members = ["server"]
resolver = "3"
```

`server/Cargo.toml`:
```toml
[package]
name = "oc2r-wiki"
version = "0.1.0"
edition = "2024"
license = "GPL-3.0-or-later"

[dependencies]
axum = "0.8"
tokio = { version = "1", features = ["rt-multi-thread", "macros", "net", "signal"] }
tracing = "0.1"
tracing-subscriber = "0.3"
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[dev-dependencies]
tower = { version = "0.5", features = ["util"] }
http-body-util = "0.1"
```

`.gitignore`:
```
/target
/web/node_modules
/web/dist
*.db
*.db-wal
*.db-shm
```

- [ ] **Step 2: Написать падающий тест health**

В конце `server/src/lib.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    #[tokio::test]
    async fn health_returns_ok() {
        let res = router()
            .oneshot(Request::get("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
    }
}
```

- [ ] **Step 3: Запустить, убедиться что не компилируется**

Run: `cargo test -p oc2r-wiki health_returns_ok`
Expected: FAIL, `cannot find function router`.

- [ ] **Step 4: Реализовать**

`server/src/lib.rs` (над тестами):
```rust
pub mod config;

use axum::{Router, routing::get};

pub fn router() -> Router {
    Router::new().route("/health", get(|| async { "ok" }))
}
```

`server/src/config.rs`:
```rust
pub struct Config {
    pub bind: String,
    pub db_path: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            bind: std::env::var("WIKI_BIND").unwrap_or_else(|_| "127.0.0.1:8080".into()),
            db_path: std::env::var("WIKI_DB").unwrap_or_else(|_| "wiki.db".into()),
        }
    }
}
```

`server/src/main.rs`:
```rust
use oc2r_wiki::{config::Config, router};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let cfg = Config::from_env();
    let listener = tokio::net::TcpListener::bind(&cfg.bind).await.expect("bind");
    tracing::info!("listening on {}", cfg.bind);
    axum::serve(listener, router()).await.expect("serve");
}
```

- [ ] **Step 5: Запустить тест**

Run: `cargo test -p oc2r-wiki health_returns_ok`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml .gitignore server
git commit -m "Add server skeleton with health endpoint"
```

---

### Task 1: AppError

**Files:**
- Create: `server/src/error.rs`
- Modify: `server/src/lib.rs` (добавить `pub mod error;`)

**Interfaces:**
- Produces:
```rust
pub enum AppError { Unauthorized, Forbidden, NotFound, Invalid(String), RateLimited(u64), Internal(String) }
impl IntoResponse for AppError   // JSON {"error","message"} + status; RateLimited добавляет Retry-After
pub type AppResult<T> = Result<T, AppError>;
```

- [ ] **Step 1: Падающие тесты** (в `error.rs`)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;
    use http_body_util::BodyExt;

    async fn body(e: AppError) -> (u16, serde_json::Value) {
        let res = e.into_response();
        let status = res.status().as_u16();
        let bytes = res.into_body().collect().await.unwrap().to_bytes();
        (status, serde_json::from_slice(&bytes).unwrap())
    }

    #[tokio::test]
    async fn maps_status_and_code() {
        assert_eq!(body(AppError::Unauthorized).await.0, 401);
        assert_eq!(body(AppError::Forbidden).await.0, 403);
        let (s, j) = body(AppError::NotFound).await;
        assert_eq!((s, j["error"].as_str()), (404, Some("not_found")));
        assert_eq!(body(AppError::Invalid("x".into())).await.0, 422);
    }

    #[test]
    fn rate_limited_sets_retry_after() {
        let res = AppError::RateLimited(30).into_response();
        assert_eq!(res.status(), 429);
        assert_eq!(res.headers()["retry-after"], "30");
    }
}
```

- [ ] **Step 2: Запустить**

Run: `cargo test -p oc2r-wiki error::`
Expected: FAIL (тип не определён).

- [ ] **Step 3: Реализовать**

```rust
use axum::{Json, http::StatusCode, response::{IntoResponse, Response}};
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    Unauthorized,
    Forbidden,
    NotFound,
    Invalid(String),
    RateLimited(u64),
    Internal(String),
}

pub type AppResult<T> = Result<T, AppError>;

impl AppError {
    fn parts(&self) -> (StatusCode, &'static str, String) {
        match self {
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized", "Нужен вход или ключ".into()),
            Self::Forbidden => (StatusCode::FORBIDDEN, "forbidden", "Недостаточно прав".into()),
            Self::NotFound => (StatusCode::NOT_FOUND, "not_found", "Не найдено".into()),
            Self::Invalid(m) => (StatusCode::UNPROCESSABLE_ENTITY, "invalid", m.clone()),
            Self::RateLimited(_) => (StatusCode::TOO_MANY_REQUESTS, "rate_limited", "Слишком часто".into()),
            Self::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal", "Внутренняя ошибка".into()),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        if let Self::Internal(m) = &self {
            tracing::error!("internal error: {m}");
        }
        let (status, code, message) = self.parts();
        let mut res = (status, Json(json!({ "error": code, "message": message }))).into_response();
        if let Self::RateLimited(secs) = self {
            res.headers_mut().insert("retry-after", secs.to_string().parse().unwrap());
        }
        res
    }
}
```

- [ ] **Step 4: Запустить**

Run: `cargo test -p oc2r-wiki error::`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add server/src/error.rs server/src/lib.rs
git commit -m "Add AppError with JSON responses"
```

---

### Task 2: domain (типы, правила, порты)

**Files:**
- Create: `server/src/domain/mod.rs`, `page.rs`, `revision.rs`, `proposal.rs`
- Create: `server/src/domain/ports/mod.rs`, `pages.rs`, `proposals.rs`, `access.rs`

**Interfaces:**
- Produces (используют все следующие задачи):
```rust
// page.rs
pub enum Lang { En, Ru, Zh }                    // Lang::parse(&str)->Option<Lang>, .code()->&'static str
pub struct Slug(String);                        // Slug::parse(&str)->Option<Slug>, .as_str()
pub struct Page { pub id: i64, pub slug: Slug, pub lang: Lang, pub section: String, pub position: i64, pub deleted: bool, pub current: Revision }
// revision.rs
pub enum Author { User(String), ApiKey(String), Import }   // .label() -> String
pub struct Revision { pub id: i64, pub page_id: i64, pub body: String, pub summary: String, pub author: Author, pub created_at: i64 }
pub struct NewRevision { pub body: String, pub summary: String, pub author: Author }
pub fn rollback_to(target: &Revision, author: Author) -> NewRevision
pub fn is_noop(current: Option<&Revision>, body: &str) -> bool
pub fn size_delta(old: Option<&str>, new: &str) -> i64
// proposal.rs
pub enum Status { Open, Accepted, Rejected }    // .as_str(), Status::parse
pub fn is_conflict(base_rev_id: i64, current_rev_id: i64) -> bool
pub struct Proposal { pub id: i64, pub page_id: i64, pub base_rev_id: i64, pub body: String, pub summary: String, pub user_id: i64, pub status: Status, pub created_at: i64 }
```

- [ ] **Step 1: Падающие тесты**

В `page.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_accepts_nested_lowercase() {
        assert_eq!(Slug::parse("blocks/computer").unwrap().as_str(), "blocks/computer");
        assert!(Slug::parse("quick_start-2").is_some());
    }

    #[test]
    fn slug_rejects_bad_input() {
        for bad in ["", "Bad", "a//b", "../x", "a b", "/a", "a/"] {
            assert!(Slug::parse(bad).is_none(), "{bad}");
        }
        assert!(Slug::parse(&"a".repeat(101)).is_none());
    }

    #[test]
    fn lang_roundtrip() {
        for c in ["en", "ru", "zh"] {
            assert_eq!(Lang::parse(c).unwrap().code(), c);
        }
        assert!(Lang::parse("de").is_none());
    }
}
```

В `revision.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn rev(id: i64, body: &str) -> Revision {
        Revision { id, page_id: 1, body: body.into(), summary: String::new(), author: Author::Import, created_at: 0 }
    }

    #[test]
    fn rollback_copies_body_and_names_target() {
        let n = rollback_to(&rev(12, "old"), Author::User("loki".into()));
        assert_eq!(n.body, "old");
        assert_eq!(n.summary, "Откат к №12");
    }

    #[test]
    fn noop_only_when_body_equal() {
        assert!(is_noop(Some(&rev(1, "a")), "a"));
        assert!(!is_noop(Some(&rev(1, "a")), "b"));
        assert!(!is_noop(None, "a"));
    }

    #[test]
    fn delta_counts_chars_not_bytes() {
        assert_eq!(size_delta(None, "привет"), 6);
        assert_eq!(size_delta(Some("abc"), "a"), -2);
    }

    #[test]
    fn author_label() {
        assert_eq!(Author::ApiKey("ci".into()).label(), "ключ: ci");
        assert_eq!(Author::User("loki".into()).label(), "loki");
    }
}
```

В `proposal.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conflict_when_current_is_newer() {
        assert!(!is_conflict(5, 5));
        assert!(is_conflict(5, 7));
    }

    #[test]
    fn status_roundtrip() {
        for s in [Status::Open, Status::Accepted, Status::Rejected] {
            assert_eq!(Status::parse(s.as_str()), Some(s));
        }
    }
}
```

- [ ] **Step 2: Запустить**

Run: `cargo test -p oc2r-wiki domain::`
Expected: FAIL (типов нет).

- [ ] **Step 3: Реализовать**

`domain/page.rs` (над тестами):
```rust
use super::revision::Revision;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Lang {
    En,
    Ru,
    Zh,
}

impl Lang {
    pub fn parse(s: &str) -> Option<Lang> {
        match s {
            "en" => Some(Lang::En),
            "ru" => Some(Lang::Ru),
            "zh" => Some(Lang::Zh),
            _ => None,
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Ru => "ru",
            Lang::Zh => "zh",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Slug(String);

impl Slug {
    pub fn parse(s: &str) -> Option<Slug> {
        let segment_ok = |seg: &str| {
            !seg.is_empty()
                && seg.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_')
        };
        (s.len() <= 100 && s.split('/').all(segment_ok)).then(|| Slug(s.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct Page {
    pub id: i64,
    pub slug: Slug,
    pub lang: Lang,
    pub section: String,
    pub position: i64,
    pub deleted: bool,
    pub current: Revision,
}
```

`domain/revision.rs` (над тестами):
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Author {
    User(String),
    ApiKey(String),
    Import,
}

impl Author {
    pub fn label(&self) -> String {
        match self {
            Author::User(login) => login.clone(),
            Author::ApiKey(name) => format!("ключ: {name}"),
            Author::Import => "импорт".into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Revision {
    pub id: i64,
    pub page_id: i64,
    pub body: String,
    pub summary: String,
    pub author: Author,
    pub created_at: i64,
}

#[derive(Debug, Clone)]
pub struct NewRevision {
    pub body: String,
    pub summary: String,
    pub author: Author,
}

pub fn rollback_to(target: &Revision, author: Author) -> NewRevision {
    NewRevision { body: target.body.clone(), summary: format!("Откат к №{}", target.id), author }
}

pub fn is_noop(current: Option<&Revision>, body: &str) -> bool {
    current.is_some_and(|c| c.body == body)
}

pub fn size_delta(old: Option<&str>, new: &str) -> i64 {
    new.chars().count() as i64 - old.map_or(0, |o| o.chars().count() as i64)
}
```

`domain/proposal.rs` (над тестами):
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Open,
    Accepted,
    Rejected,
}

impl Status {
    pub fn as_str(self) -> &'static str {
        match self {
            Status::Open => "open",
            Status::Accepted => "accepted",
            Status::Rejected => "rejected",
        }
    }

    pub fn parse(s: &str) -> Option<Status> {
        match s {
            "open" => Some(Status::Open),
            "accepted" => Some(Status::Accepted),
            "rejected" => Some(Status::Rejected),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Proposal {
    pub id: i64,
    pub page_id: i64,
    pub base_rev_id: i64,
    pub body: String,
    pub summary: String,
    pub user_id: i64,
    pub status: Status,
    pub created_at: i64,
}

pub fn is_conflict(base_rev_id: i64, current_rev_id: i64) -> bool {
    base_rev_id != current_rev_id
}
```

`domain/mod.rs`:
```rust
pub mod page;
pub mod ports;
pub mod proposal;
pub mod revision;
```

`domain/ports/mod.rs`:
```rust
pub mod access;
pub mod pages;
pub mod proposals;

#[derive(Debug)]
pub struct StoreError(pub String);

impl From<StoreError> for crate::error::AppError {
    fn from(e: StoreError) -> Self {
        crate::error::AppError::Internal(e.0)
    }
}

pub type StoreResult<T> = Result<T, StoreError>;
```

`domain/ports/pages.rs`:
```rust
use super::StoreResult;
use crate::domain::{page::{Lang, Page, Slug}, revision::{NewRevision, Revision}};

#[derive(Debug, PartialEq, Eq)]
pub enum SaveOutcome {
    Created(i64),
    Updated(i64),
    Unchanged,
}

pub struct PageSummary {
    pub slug: Slug,
    pub lang: Lang,
    pub section: String,
    pub position: i64,
    pub title_rev_id: i64,
}

pub struct SearchHit {
    pub slug: Slug,
    pub lang: Lang,
    pub snippet: String,
}

pub trait PageRepo: Send + Sync {
    fn find(&self, slug: &Slug, lang: Lang) -> StoreResult<Option<Page>>;
    fn revision(&self, page_id: i64, rev_id: i64) -> StoreResult<Option<Revision>>;
    fn history(&self, page_id: i64) -> StoreResult<Vec<Revision>>;
    fn save(&self, slug: &Slug, lang: Lang, rev: NewRevision) -> StoreResult<SaveOutcome>;
    fn list(&self) -> StoreResult<Vec<PageSummary>>;
    fn set_deleted(&self, page_id: i64, deleted: bool) -> StoreResult<()>;
    fn search(&self, query: &str, lang: Lang) -> StoreResult<Vec<SearchHit>>;
}
```

`domain/ports/proposals.rs`:
```rust
use super::StoreResult;
use crate::domain::proposal::{Proposal, Status};

pub trait ProposalRepo: Send + Sync {
    fn create(&self, page_id: i64, base_rev_id: i64, body: &str, summary: &str, user_id: i64) -> StoreResult<i64>;
    fn get(&self, id: i64) -> StoreResult<Option<Proposal>>;
    fn list(&self, status: Status) -> StoreResult<Vec<Proposal>>;
    fn count_open_by_user(&self, user_id: i64) -> StoreResult<u32>;
    fn count_since_by_user(&self, user_id: i64, since: i64) -> StoreResult<u32>;
    fn decide(&self, id: i64, status: Status, admin_id: i64, note: &str) -> StoreResult<()>;
}
```

`domain/ports/access.rs`:
```rust
use super::StoreResult;

#[derive(Debug, Clone)]
pub struct UserRow {
    pub id: i64,
    pub github_id: i64,
    pub login: String,
    pub admin: bool,
    pub blocked: bool,
}

#[derive(Debug, Clone)]
pub struct KeyRow {
    pub id: i64,
    pub name: String,
    pub read: bool,
    pub write: bool,
    pub export: bool,
}

pub trait AccessRepo: Send + Sync {
    fn upsert_user(&self, github_id: i64, login: &str) -> StoreResult<UserRow>;
    fn user(&self, id: i64) -> StoreResult<Option<UserRow>>;
    fn set_blocked(&self, id: i64, blocked: bool) -> StoreResult<()>;
    fn set_admin(&self, id: i64, admin: bool) -> StoreResult<()>;
    fn create_key(&self, name: &str, hash: &str, read: bool, write: bool, export: bool) -> StoreResult<i64>;
    fn key_by_hash(&self, hash: &str) -> StoreResult<Option<KeyRow>>;
    fn touch_key(&self, id: i64) -> StoreResult<()>;
    fn revoke_key(&self, id: i64) -> StoreResult<()>;
    fn list_keys(&self) -> StoreResult<Vec<KeyRow>>;
}
```

Добавить `pub mod domain;` в `server/src/lib.rs`.

- [ ] **Step 4: Запустить**

Run: `cargo test -p oc2r-wiki domain::`
Expected: PASS (все 9 тестов).

- [ ] **Step 5: Commit**

```bash
git add server/src/domain server/src/lib.rs
git commit -m "Add domain types, rules and repository ports"
```

---

### Task 3: diff

**Files:**
- Create: `server/src/diff/mod.rs`, `server/src/diff/lines.rs`

**Interfaces:**
- Produces: `diff::lines::{Line<'a>, diff_lines}` и реэкспорт в `diff::`.
```rust
pub enum Line<'a> { Same(&'a str), Added(&'a str), Removed(&'a str) }
pub fn diff_lines<'a>(old: &'a str, new: &'a str) -> Vec<Line<'a>>
```

- [ ] **Step 1: Падающие тесты** (в `lines.rs`)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_is_all_same() {
        assert_eq!(diff_lines("a\nb", "a\nb"), vec![Line::Same("a"), Line::Same("b")]);
    }

    #[test]
    fn changed_line_is_removed_then_added() {
        assert_eq!(
            diff_lines("a\nb\nc", "a\nX\nc"),
            vec![Line::Same("a"), Line::Removed("b"), Line::Added("X"), Line::Same("c")]
        );
    }

    #[test]
    fn appended_and_deleted_lines() {
        assert_eq!(diff_lines("a", "a\nb"), vec![Line::Same("a"), Line::Added("b")]);
        assert_eq!(diff_lines("a\nb", "a"), vec![Line::Same("a"), Line::Removed("b")]);
    }

    #[test]
    fn empty_old_is_all_added() {
        assert_eq!(diff_lines("", "a"), vec![Line::Added("a")]);
    }
}
```

- [ ] **Step 2: Запустить** — `cargo test -p oc2r-wiki diff::` → FAIL.

- [ ] **Step 3: Реализовать**

`diff/lines.rs` (над тестами):
```rust
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Line<'a> {
    Same(&'a str),
    Added(&'a str),
    Removed(&'a str),
}

pub fn diff_lines<'a>(old: &'a str, new: &'a str) -> Vec<Line<'a>> {
    let a: Vec<&str> = old.lines().collect();
    let b: Vec<&str> = new.lines().collect();
    let (n, m) = (a.len(), b.len());

    // t[i][j] — длина наибольшей общей подпоследовательности a[i..] и b[j..]
    let mut t = vec![vec![0u32; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            t[i][j] = if a[i] == b[j] { t[i + 1][j + 1] + 1 } else { t[i + 1][j].max(t[i][j + 1]) };
        }
    }

    let (mut i, mut j) = (0, 0);
    let mut out = Vec::new();
    while i < n && j < m {
        if a[i] == b[j] {
            out.push(Line::Same(a[i]));
            i += 1;
            j += 1;
        } else if t[i + 1][j] >= t[i][j + 1] {
            out.push(Line::Removed(a[i]));
            i += 1;
        } else {
            out.push(Line::Added(b[j]));
            j += 1;
        }
    }
    out.extend(a[i..].iter().map(|l| Line::Removed(l)));
    out.extend(b[j..].iter().map(|l| Line::Added(l)));
    out
}
```

`diff/mod.rs`:
```rust
pub mod lines;

pub use lines::{Line, diff_lines};
```

Добавить `pub mod diff;` в `server/src/lib.rs`.

- [ ] **Step 4: Запустить** — `cargo test -p oc2r-wiki diff::` → PASS.

- [ ] **Step 5: Commit**

```bash
git add server/src/diff server/src/lib.rs
git commit -m "Add line diff"
```

---

### Task 4: render (Markdown → безопасный HTML)

**Files:**
- Create: `server/src/render/mod.rs`, `markdown.rs`, `extensions.rs`, `sanitize.rs`
- Modify: `server/Cargo.toml` (добавить `pulldown-cmark = "0.13"`, `ammonia = "4"`)

**Interfaces:**
- Produces: `render::to_html(src: &str) -> String` — единственный путь рендера.
- Внутренние: `extensions::expand(src: &str) -> String`, `sanitize::clean(html: &str) -> String`.

Расширения: `:::tip`, `:::warn`, `:::version 26.x`, `:::recipe` — закрываются строкой `:::`. В рецепте каждая строка — один слот (до 9), пустая строка — пустой слот.

- [ ] **Step 1: Падающие тесты**

В `extensions.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tip_becomes_callout_div() {
        let out = expand(":::tip\nТекст\n:::");
        assert!(out.contains("<div class=\"callout tip\">"));
        assert!(out.contains("</div>"));
        assert!(out.contains("Текст"));
    }

    #[test]
    fn version_names_the_version() {
        let out = expand(":::version 26.x\nТолько тут\n:::");
        assert!(out.contains("callout version"));
        assert!(out.contains("26.x"));
    }

    #[test]
    fn recipe_has_nine_slots() {
        let out = expand(":::recipe\niron\n\niron\n:::");
        assert_eq!(out.matches("class=\"slot\"").count(), 9);
        assert!(out.contains(">iron<"));
    }

    #[test]
    fn plain_text_untouched() {
        assert_eq!(expand("hello\nworld"), "hello\nworld");
    }
}
```

В `markdown.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_basic_markdown() {
        let html = to_html("# Title\n\n**bold**");
        assert!(html.contains("<h1>Title</h1>"));
        assert!(html.contains("<strong>bold</strong>"));
    }

    #[test]
    fn strips_script_and_handlers() {
        let html = to_html("<script>alert(1)</script><img src=x onerror=alert(1)>hi");
        assert!(!html.contains("script"));
        assert!(!html.contains("onerror"));
        assert!(html.contains("hi"));
    }

    #[test]
    fn keeps_callout_classes_but_not_arbitrary_ones() {
        let html = to_html(":::tip\nok\n:::");
        assert!(html.contains("class=\"callout tip\""));
        let evil = to_html("<div class=\"evil\" style=\"x\">a</div>");
        assert!(!evil.contains("evil") && !evil.contains("style"));
    }
}
```

- [ ] **Step 2: Запустить** — `cargo test -p oc2r-wiki render::` → FAIL.

- [ ] **Step 3: Реализовать**

`render/extensions.rs` (над тестами):
```rust
/// Раскрывает блоки `:::имя [аргумент]` ... `:::` в HTML-контейнеры.
/// Пустые строки вокруг внутреннего Markdown нужны, чтобы он разбирался внутри `<div>`.
pub fn expand(src: &str) -> String {
    let mut out = Vec::new();
    let mut lines = src.lines();
    while let Some(line) = lines.next() {
        let Some(head) = line.strip_prefix(":::") else {
            out.push(line.to_owned());
            continue;
        };
        let (name, arg) = head.split_once(' ').map_or((head, ""), |(n, a)| (n, a.trim()));
        let body: Vec<&str> = lines.by_ref().take_while(|l| *l != ":::").collect();
        match name {
            "tip" | "warn" => wrap(&mut out, &format!("callout {name}"), None, &body),
            "version" => wrap(&mut out, "callout version", Some(&format!("**Только для {arg}**")), &body),
            "recipe" => recipe(&mut out, &body),
            _ => out.extend(body.iter().map(|l| (*l).to_owned())),
        }
    }
    out.join("\n")
}

fn wrap(out: &mut Vec<String>, class: &str, title: Option<&str>, body: &[&str]) {
    out.push(format!("<div class=\"{class}\">\n"));
    if let Some(t) = title {
        out.push(format!("{t}\n"));
    }
    out.extend(body.iter().map(|l| (*l).to_owned()));
    out.push("\n</div>".into());
}

fn recipe(out: &mut Vec<String>, body: &[&str]) {
    out.push("<div class=\"recipe\">".into());
    for i in 0..9 {
        let item = body.get(i).map_or("", |s| s.trim());
        out.push(format!("<div class=\"slot\">{item}</div>"));
    }
    out.push("</div>".into());
}
```
Название слота идёт в HTML, поэтому оно проходит через `sanitize::clean` как весь остальной вывод.

`render/sanitize.rs`:
```rust
use ammonia::Builder;

pub fn clean(html: &str) -> String {
    let mut b = Builder::default();
    b.add_tags(["div"]);
    b.add_allowed_classes("div", ["callout", "tip", "warn", "version", "recipe", "slot"]);
    b.clean(html).to_string()
}
```

`render/markdown.rs` (над тестами):
```rust
use super::{extensions, sanitize};
use pulldown_cmark::{Options, Parser, html};

pub fn to_html(src: &str) -> String {
    let expanded = extensions::expand(src);
    let opts = Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_HEADING_ATTRIBUTES;
    let mut raw = String::new();
    html::push_html(&mut raw, Parser::new_ext(&expanded, opts));
    sanitize::clean(&raw)
}
```

`render/mod.rs`:
```rust
mod extensions;
mod markdown;
mod sanitize;

pub use markdown::to_html;
```

Добавить `pub mod render;` в `server/src/lib.rs`.

- [ ] **Step 4: Запустить** — `cargo test -p oc2r-wiki render::` → PASS. Если ammonia отклоняет `class="callout tip"` (порядок или нормализация классов), подправить `add_allowed_classes` и оставить тест как эталон поведения.

- [ ] **Step 5: Commit**

```bash
git add server/Cargo.toml server/src/render server/src/lib.rs
git commit -m "Add Markdown render pipeline with extensions and sanitizing"
```

---

### Task 5: policy и API-ключи

**Files:**
- Create: `server/src/auth/mod.rs`, `server/src/auth/policy.rs`
- Create: `server/src/auth/tokens/mod.rs`, `server/src/auth/tokens/api_key.rs`
- Modify: `server/Cargo.toml` (добавить `sha2 = "0.10"`, `rand = "0.9"`)

**Interfaces:**
- Produces:
```rust
// policy.rs
pub enum Actor { Anonymous, User { id: i64, admin: bool, blocked: bool }, Key { id: i64, read: bool, write: bool, export: bool } }
pub enum Action { ReadPage, Propose, WritePage, DecideProposal, ManageKeys, ManageUsers, Export }
pub fn can(actor: &Actor, action: Action) -> bool
// api_key.rs
pub fn generate() -> (String, String)   // (ключ для показа, хеш для базы)
pub fn hash(key: &str) -> String
```

- [ ] **Step 1: Падающие тесты**

В `policy.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn user(admin: bool, blocked: bool) -> Actor {
        Actor::User { id: 1, admin, blocked }
    }

    fn key(read: bool, write: bool, export: bool) -> Actor {
        Actor::Key { id: 1, read, write, export }
    }

    #[test]
    fn anyone_can_read() {
        for a in [Actor::Anonymous, user(false, false), key(false, false, false)] {
            assert!(can(&a, Action::ReadPage));
        }
    }

    #[test]
    fn only_signed_in_unblocked_can_propose() {
        assert!(!can(&Actor::Anonymous, Action::Propose));
        assert!(can(&user(false, false), Action::Propose));
        assert!(!can(&user(false, true), Action::Propose));
    }

    #[test]
    fn writing_needs_admin_or_write_key() {
        assert!(!can(&user(false, false), Action::WritePage));
        assert!(can(&user(true, false), Action::WritePage));
        assert!(can(&key(true, true, false), Action::WritePage));
        assert!(!can(&key(true, false, false), Action::WritePage));
    }

    #[test]
    fn admin_only_actions() {
        for act in [Action::DecideProposal, Action::ManageKeys, Action::ManageUsers] {
            assert!(can(&user(true, false), act));
            assert!(!can(&user(false, false), act));
            assert!(!can(&key(true, true, true), act));
            assert!(!can(&user(true, true), act));
        }
    }

    #[test]
    fn export_for_admin_or_export_key() {
        assert!(can(&user(true, false), Action::Export));
        assert!(can(&key(false, false, true), Action::Export));
        assert!(!can(&key(true, true, false), Action::Export));
    }
}
```

В `api_key.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_key_matches_its_hash() {
        let (key, h) = generate();
        assert!(key.starts_with("ocw_") && key.len() == 4 + 64);
        assert_eq!(hash(&key), h);
        assert_eq!(h.len(), 64);
    }

    #[test]
    fn keys_are_unique() {
        assert_ne!(generate().0, generate().0);
    }

    #[test]
    fn hash_is_deterministic_and_differs_per_input() {
        assert_eq!(hash("a"), hash("a"));
        assert_ne!(hash("a"), hash("b"));
    }
}
```

- [ ] **Step 2: Запустить** — `cargo test -p oc2r-wiki auth::` → FAIL.

- [ ] **Step 3: Реализовать**

`auth/policy.rs` (над тестами):
```rust
#[derive(Debug, Clone)]
pub enum Actor {
    Anonymous,
    User { id: i64, admin: bool, blocked: bool },
    Key { id: i64, read: bool, write: bool, export: bool },
}

#[derive(Debug, Clone, Copy)]
pub enum Action {
    ReadPage,
    Propose,
    WritePage,
    DecideProposal,
    ManageKeys,
    ManageUsers,
    Export,
}

pub fn can(actor: &Actor, action: Action) -> bool {
    use Action::*;
    match (actor, action) {
        (_, ReadPage) => true,
        (Actor::User { blocked: false, .. }, Propose) => true,
        (Actor::User { admin: true, blocked: false, .. }, _) => true,
        (Actor::Key { write: true, .. }, WritePage) => true,
        (Actor::Key { export: true, .. }, Export) => true,
        _ => false,
    }
}
```

`auth/tokens/api_key.rs` (над тестами):
```rust
use rand::RngCore;
use sha2::{Digest, Sha256};

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn hash(key: &str) -> String {
    hex(&Sha256::digest(key.as_bytes()))
}

pub fn generate() -> (String, String) {
    let mut raw = [0u8; 32];
    rand::rng().fill_bytes(&mut raw);
    let key = format!("ocw_{}", hex(&raw));
    let h = hash(&key);
    (key, h)
}
```

`auth/tokens/mod.rs`:
```rust
pub mod api_key;
```

`auth/mod.rs`:
```rust
pub mod policy;
pub mod tokens;
```

Добавить `pub mod auth;` в `server/src/lib.rs`.

- [ ] **Step 4: Запустить** — `cargo test -p oc2r-wiki auth::` → PASS.

- [ ] **Step 5: Commit**

```bash
git add server/Cargo.toml server/src/auth server/src/lib.rs
git commit -m "Add access policy and API key hashing"
```

---

### Task 6: store — база и миграции

**Files:**
- Create: `server/src/store/mod.rs`, `server/src/store/db.rs`
- Create: `server/src/store/sql/0001_content.sql`, `0002_access.sql`, `0003_search.sql`
- Modify: `server/Cargo.toml` (добавить `rusqlite = { version = "0.37", features = ["bundled"] }`)

**Interfaces:**
- Produces:
```rust
pub type Db = std::sync::Arc<std::sync::Mutex<rusqlite::Connection>>;
pub fn open(path: &str) -> StoreResult<Db>          // WAL, foreign_keys, миграции
pub fn open_memory() -> StoreResult<Db>             // для тестов
pub fn err(e: rusqlite::Error) -> StoreError
pub fn now() -> i64                                 // unix seconds
```

- [ ] **Step 1: Падающий тест** (в `db.rs`)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_create_all_tables() {
        let db = open_memory().unwrap();
        let conn = db.lock().unwrap();
        for table in ["pages", "revisions", "proposals", "users", "api_keys", "media", "revisions_fts"] {
            let n: i64 = conn
                .query_row("select count(*) from sqlite_master where name = ?1", [table], |r| r.get(0))
                .unwrap();
            assert_eq!(n, 1, "{table}");
        }
    }

    #[test]
    fn migrations_are_idempotent() {
        let db = open_memory().unwrap();
        let conn = db.lock().unwrap();
        migrate(&conn).unwrap();
    }
}
```

- [ ] **Step 2: Запустить** — `cargo test -p oc2r-wiki store::db` → FAIL.

- [ ] **Step 3: Реализовать SQL**

`0001_content.sql`:
```sql
create table pages (
  id integer primary key,
  slug text not null,
  lang text not null,
  section text not null default '',
  position integer not null default 0,
  current_rev_id integer,
  deleted_at integer,
  created_at integer not null
);
create unique index pages_slug_lang on pages(slug, lang) where deleted_at is null;

create table revisions (
  id integer primary key,
  page_id integer not null references pages(id),
  body text not null,
  summary text not null default '',
  author_kind text not null check (author_kind in ('user','api_key','import')),
  author_ref text not null default '',
  created_at integer not null
);
create index revisions_page on revisions(page_id, id desc);

create table media (
  id integer primary key,
  filename text not null,
  mime text not null,
  size integer not null,
  sha256 text not null,
  created_at integer not null
);
```

`0002_access.sql`:
```sql
create table users (
  id integer primary key,
  github_id integer not null unique,
  login text not null,
  admin integer not null default 0,
  blocked_at integer,
  created_at integer not null
);

create table api_keys (
  id integer primary key,
  name text not null,
  key_hash text not null unique,
  can_read integer not null default 1,
  can_write integer not null default 0,
  can_export integer not null default 0,
  created_at integer not null,
  last_used_at integer,
  revoked_at integer
);

create table proposals (
  id integer primary key,
  page_id integer not null references pages(id),
  base_rev_id integer not null references revisions(id),
  body text not null,
  summary text not null default '',
  user_id integer not null references users(id),
  status text not null default 'open' check (status in ('open','accepted','rejected')),
  decided_by integer references users(id),
  decided_at integer,
  decision_note text not null default '',
  created_at integer not null
);
create index proposals_status on proposals(status, id);
create index proposals_user on proposals(user_id, created_at);
```

`0003_search.sql`:
```sql
create virtual table revisions_fts using fts5(slug, body, page_id unindexed, tokenize = 'unicode61');
```
Индекс заполняется кодом сохранения страницы (Task 7), одним местом, без триггеров: в нём всегда лежит только текущая ревизия страницы.

- [ ] **Step 4: Реализовать `db.rs`** (над тестами)

```rust
use crate::domain::ports::{StoreError, StoreResult};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

pub type Db = Arc<Mutex<Connection>>;

const MIGRATIONS: [&str; 3] = [
    include_str!("sql/0001_content.sql"),
    include_str!("sql/0002_access.sql"),
    include_str!("sql/0003_search.sql"),
];

pub fn err(e: rusqlite::Error) -> StoreError {
    StoreError(e.to_string())
}

pub fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64)
}

pub fn migrate(conn: &Connection) -> StoreResult<()> {
    let applied: usize = conn.pragma_query_value(None, "user_version", |r| r.get(0)).map_err(err)?;
    for (i, sql) in MIGRATIONS.iter().enumerate().skip(applied) {
        conn.execute_batch(sql).map_err(err)?;
        conn.pragma_update(None, "user_version", i + 1).map_err(err)?;
    }
    Ok(())
}

fn setup(conn: Connection) -> StoreResult<Db> {
    conn.pragma_update(None, "foreign_keys", "ON").map_err(err)?;
    migrate(&conn)?;
    Ok(Arc::new(Mutex::new(conn)))
}

pub fn open(path: &str) -> StoreResult<Db> {
    let conn = Connection::open(path).map_err(err)?;
    conn.pragma_update(None, "journal_mode", "WAL").map_err(err)?;
    setup(conn)
}

pub fn open_memory() -> StoreResult<Db> {
    setup(Connection::open_in_memory().map_err(err)?)
}
```

`store/mod.rs`:
```rust
pub mod db;
```

Добавить `pub mod store;` в `server/src/lib.rs`.

- [ ] **Step 5: Запустить** — `cargo test -p oc2r-wiki store::db` → PASS.

- [ ] **Step 6: Commit**

```bash
git add server/Cargo.toml server/src/store server/src/lib.rs
git commit -m "Add SQLite setup with migrations"
```

---

### Task 7: store/content — страницы, ревизии, поиск

**Files:**
- Create: `server/src/store/content/mod.rs`, `pages.rs`, `revisions.rs`, `search.rs`
- Modify: `server/src/store/mod.rs` (`pub mod content;`)

**Interfaces:**
- Consumes: `Db`, `err`, `now` (Task 6); `PageRepo`, `SaveOutcome`, `PageSummary`, `SearchHit` (Task 2).
- Produces: `content::SqlitePages { db: Db }` с `SqlitePages::new(db: Db) -> Self`, реализует `PageRepo`.

Разделение: `pages.rs` — `impl PageRepo` (делегирует), `revisions.rs` — чтение и вставка ревизий, `search.rs` — запись и запрос индекса.

- [ ] **Step 1: Падающие тесты** (в `content/mod.rs`)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{page::{Lang, Slug}, revision::{Author, NewRevision, rollback_to}};
    use crate::domain::ports::pages::{PageRepo, SaveOutcome};
    use crate::store::db::open_memory;

    fn repo() -> SqlitePages {
        SqlitePages::new(open_memory().unwrap())
    }

    fn rev(body: &str) -> NewRevision {
        NewRevision { body: body.into(), summary: "s".into(), author: Author::User("loki".into()) }
    }

    fn slug(s: &str) -> Slug {
        Slug::parse(s).unwrap()
    }

    #[test]
    fn save_creates_then_updates_then_noops() {
        let r = repo();
        assert!(matches!(r.save(&slug("a"), Lang::En, rev("one")).unwrap(), SaveOutcome::Created(_)));
        assert!(matches!(r.save(&slug("a"), Lang::En, rev("two")).unwrap(), SaveOutcome::Updated(_)));
        assert_eq!(r.save(&slug("a"), Lang::En, rev("two")).unwrap(), SaveOutcome::Unchanged);
        let page = r.find(&slug("a"), Lang::En).unwrap().unwrap();
        assert_eq!(page.current.body, "two");
        assert_eq!(r.history(page.id).unwrap().len(), 2);
    }

    #[test]
    fn old_revision_stays_readable() {
        let r = repo();
        r.save(&slug("a"), Lang::En, rev("one")).unwrap();
        r.save(&slug("a"), Lang::En, rev("two")).unwrap();
        let page = r.find(&slug("a"), Lang::En).unwrap().unwrap();
        let first = r.history(page.id).unwrap().last().unwrap().clone();
        assert_eq!(r.revision(page.id, first.id).unwrap().unwrap().body, "one");
    }

    #[test]
    fn rollback_adds_a_new_revision() {
        let r = repo();
        r.save(&slug("a"), Lang::En, rev("one")).unwrap();
        r.save(&slug("a"), Lang::En, rev("two")).unwrap();
        let page = r.find(&slug("a"), Lang::En).unwrap().unwrap();
        let first = r.history(page.id).unwrap().last().unwrap().clone();
        r.save(&slug("a"), Lang::En, rollback_to(&first, Author::User("loki".into()))).unwrap();
        let now = r.find(&slug("a"), Lang::En).unwrap().unwrap();
        assert_eq!(now.current.body, "one");
        assert_eq!(r.history(page.id).unwrap().len(), 3);
    }

    #[test]
    fn languages_are_separate_pages() {
        let r = repo();
        r.save(&slug("a"), Lang::En, rev("en")).unwrap();
        assert!(r.find(&slug("a"), Lang::Ru).unwrap().is_none());
    }

    #[test]
    fn deleted_pages_are_hidden_and_restorable() {
        let r = repo();
        r.save(&slug("a"), Lang::En, rev("x")).unwrap();
        let id = r.find(&slug("a"), Lang::En).unwrap().unwrap().id;
        r.set_deleted(id, true).unwrap();
        assert!(r.find(&slug("a"), Lang::En).unwrap().is_none());
        r.set_deleted(id, false).unwrap();
        assert!(r.find(&slug("a"), Lang::En).unwrap().is_some());
    }

    #[test]
    fn search_finds_current_text_only() {
        let r = repo();
        r.save(&slug("a"), Lang::En, rev("alpha beta")).unwrap();
        r.save(&slug("a"), Lang::En, rev("gamma")).unwrap();
        assert!(r.search("alpha", Lang::En).unwrap().is_empty());
        assert_eq!(r.search("gamma", Lang::En).unwrap().len(), 1);
    }
}
```

- [ ] **Step 2: Запустить** — `cargo test -p oc2r-wiki store::content` → FAIL.

- [ ] **Step 3: Реализовать**

`content/revisions.rs`:
```rust
use crate::domain::{ports::StoreResult, revision::{Author, NewRevision, Revision}};
use crate::store::db::{err, now};
use rusqlite::{Connection, Row, params};

fn author_parts(a: &Author) -> (&'static str, String) {
    match a {
        Author::User(s) => ("user", s.clone()),
        Author::ApiKey(s) => ("api_key", s.clone()),
        Author::Import => ("import", String::new()),
    }
}

fn author_from(kind: &str, r: String) -> Author {
    match kind {
        "user" => Author::User(r),
        "api_key" => Author::ApiKey(r),
        _ => Author::Import,
    }
}

pub const COLUMNS: &str = "id, page_id, body, summary, author_kind, author_ref, created_at";

pub fn from_row(row: &Row) -> rusqlite::Result<Revision> {
    Ok(Revision {
        id: row.get(0)?,
        page_id: row.get(1)?,
        body: row.get(2)?,
        summary: row.get(3)?,
        author: author_from(&row.get::<_, String>(4)?, row.get(5)?),
        created_at: row.get(6)?,
    })
}

pub fn insert(conn: &Connection, page_id: i64, rev: &NewRevision) -> StoreResult<i64> {
    let (kind, who) = author_parts(&rev.author);
    conn.execute(
        "insert into revisions (page_id, body, summary, author_kind, author_ref, created_at) values (?1,?2,?3,?4,?5,?6)",
        params![page_id, rev.body, rev.summary, kind, who, now()],
    )
    .map_err(err)?;
    Ok(conn.last_insert_rowid())
}

pub fn get(conn: &Connection, page_id: i64, rev_id: i64) -> StoreResult<Option<Revision>> {
    let sql = format!("select {COLUMNS} from revisions where page_id = ?1 and id = ?2");
    match conn.query_row(&sql, params![page_id, rev_id], from_row) {
        Ok(r) => Ok(Some(r)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(err(e)),
    }
}

pub fn history(conn: &Connection, page_id: i64) -> StoreResult<Vec<Revision>> {
    let sql = format!("select {COLUMNS} from revisions where page_id = ?1 order by id desc");
    let mut stmt = conn.prepare(&sql).map_err(err)?;
    stmt.query_map([page_id], from_row)
        .map_err(err)?
        .collect::<Result<_, _>>()
        .map_err(err)
}
```

`content/search.rs`:
```rust
use crate::domain::{page::{Lang, Slug}, ports::{StoreResult, pages::SearchHit}};
use crate::store::db::err;
use rusqlite::{Connection, params};

pub fn index(conn: &Connection, page_id: i64, slug: &Slug, body: &str) -> StoreResult<()> {
    conn.execute("delete from revisions_fts where page_id = ?1", [page_id]).map_err(err)?;
    conn.execute(
        "insert into revisions_fts (slug, body, page_id) values (?1, ?2, ?3)",
        params![slug.as_str().replace(['/', '-', '_'], " "), body, page_id],
    )
    .map_err(err)?;
    Ok(())
}

pub fn remove(conn: &Connection, page_id: i64) -> StoreResult<()> {
    conn.execute("delete from revisions_fts where page_id = ?1", [page_id]).map_err(err)?;
    Ok(())
}

/// Оборачивает каждое слово в кавычки, чтобы пользовательский ввод не ломал синтаксис FTS5.
fn quote(query: &str) -> String {
    query.split_whitespace().map(|w| format!("\"{}\"", w.replace('"', ""))).collect::<Vec<_>>().join(" ")
}

pub fn query(conn: &Connection, q: &str, lang: Lang) -> StoreResult<Vec<SearchHit>> {
    let q = quote(q);
    if q.is_empty() {
        return Ok(Vec::new());
    }
    let mut stmt = conn
        .prepare(
            "select p.slug, p.lang, snippet(revisions_fts, 1, '', '', '…', 12)
             from revisions_fts f join pages p on p.id = f.page_id
             where revisions_fts match ?1 and p.lang = ?2 and p.deleted_at is null
             order by rank limit 20",
        )
        .map_err(err)?;
    let rows = stmt
        .query_map(params![q, lang.code()], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?))
        })
        .map_err(err)?;
    let mut hits = Vec::new();
    for row in rows {
        let (slug, lang, snippet) = row.map_err(err)?;
        if let (Some(slug), Some(lang)) = (Slug::parse(&slug), Lang::parse(&lang)) {
            hits.push(SearchHit { slug, lang, snippet });
        }
    }
    Ok(hits)
}
```

`content/pages.rs`:
```rust
use super::{revisions, search};
use crate::domain::{
    page::{Lang, Page, Slug},
    ports::{StoreResult, pages::{PageRepo, PageSummary, SaveOutcome, SearchHit}},
    revision::{self, NewRevision, Revision},
};
use crate::store::db::{Db, err, now};
use rusqlite::{Connection, OptionalExtension, params};

pub struct SqlitePages {
    db: Db,
}

impl SqlitePages {
    pub fn new(db: Db) -> Self {
        Self { db }
    }
}

fn page_id(conn: &Connection, slug: &Slug, lang: Lang) -> StoreResult<Option<(i64, Option<i64>)>> {
    conn.query_row(
        "select id, current_rev_id from pages where slug = ?1 and lang = ?2 and deleted_at is null",
        params![slug.as_str(), lang.code()],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )
    .optional()
    .map_err(err)
}

impl PageRepo for SqlitePages {
    fn find(&self, slug: &Slug, lang: Lang) -> StoreResult<Option<Page>> {
        let conn = self.db.lock().unwrap();
        let row = conn
            .query_row(
                "select id, section, position, current_rev_id from pages
                 where slug = ?1 and lang = ?2 and deleted_at is null",
                params![slug.as_str(), lang.code()],
                |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, i64>(2)?, r.get::<_, i64>(3)?)),
            )
            .optional()
            .map_err(err)?;
        let Some((id, section, position, rev_id)) = row else { return Ok(None) };
        let current = revisions::get(&conn, id, rev_id)?.ok_or_else(|| crate::domain::ports::StoreError("missing current revision".into()))?;
        Ok(Some(Page { id, slug: slug.clone(), lang, section, position, deleted: false, current }))
    }

    fn revision(&self, page_id: i64, rev_id: i64) -> StoreResult<Option<Revision>> {
        revisions::get(&self.db.lock().unwrap(), page_id, rev_id)
    }

    fn history(&self, page_id: i64) -> StoreResult<Vec<Revision>> {
        revisions::history(&self.db.lock().unwrap(), page_id)
    }

    fn save(&self, slug: &Slug, lang: Lang, rev: NewRevision) -> StoreResult<SaveOutcome> {
        let mut conn = self.db.lock().unwrap();
        let tx = conn.transaction().map_err(err)?;
        let existing = page_id(&tx, slug, lang)?;
        let (id, created) = match existing {
            Some((id, cur)) => {
                let current = match cur {
                    Some(c) => revisions::get(&tx, id, c)?,
                    None => None,
                };
                if revision::is_noop(current.as_ref(), &rev.body) {
                    return Ok(SaveOutcome::Unchanged);
                }
                (id, false)
            }
            None => {
                tx.execute(
                    "insert into pages (slug, lang, created_at) values (?1, ?2, ?3)",
                    params![slug.as_str(), lang.code(), now()],
                )
                .map_err(err)?;
                (tx.last_insert_rowid(), true)
            }
        };
        let rev_id = revisions::insert(&tx, id, &rev)?;
        tx.execute("update pages set current_rev_id = ?1 where id = ?2", params![rev_id, id]).map_err(err)?;
        search::index(&tx, id, slug, &rev.body)?;
        tx.commit().map_err(err)?;
        Ok(if created { SaveOutcome::Created(rev_id) } else { SaveOutcome::Updated(rev_id) })
    }

    fn list(&self) -> StoreResult<Vec<PageSummary>> {
        let conn = self.db.lock().unwrap();
        let mut stmt = conn
            .prepare("select slug, lang, section, position, current_rev_id from pages where deleted_at is null order by section, position, slug")
            .map_err(err)?;
        let rows = stmt
            .query_map([], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?, r.get::<_, i64>(3)?, r.get::<_, i64>(4)?))
            })
            .map_err(err)?;
        let mut out = Vec::new();
        for row in rows {
            let (slug, lang, section, position, title_rev_id) = row.map_err(err)?;
            if let (Some(slug), Some(lang)) = (Slug::parse(&slug), Lang::parse(&lang)) {
                out.push(PageSummary { slug, lang, section, position, title_rev_id });
            }
        }
        Ok(out)
    }

    fn set_deleted(&self, page_id: i64, deleted: bool) -> StoreResult<()> {
        let conn = self.db.lock().unwrap();
        let at: Option<i64> = deleted.then(now);
        conn.execute("update pages set deleted_at = ?1 where id = ?2", params![at, page_id]).map_err(err)?;
        if deleted {
            search::remove(&conn, page_id)?;
        } else if let Ok(Some((slug, body))) = conn
            .query_row(
                "select p.slug, r.body from pages p join revisions r on r.id = p.current_rev_id where p.id = ?1",
                [page_id],
                |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
            )
            .optional()
        {
            if let Some(slug) = Slug::parse(&slug) {
                search::index(&conn, page_id, &slug, &body)?;
            }
        }
        Ok(())
    }

    fn search(&self, query: &str, lang: Lang) -> StoreResult<Vec<SearchHit>> {
        search::query(&self.db.lock().unwrap(), query, lang)
    }
}
```

`content/mod.rs` (над тестами):
```rust
mod pages;
mod revisions;
mod search;

pub use pages::SqlitePages;
```

- [ ] **Step 4: Запустить** — `cargo test -p oc2r-wiki store::content` → PASS (6 тестов). Если какой-то файл превысил 250 строк, вынести `list` и `set_deleted` в `store/content/listing.rs`, но в папке остаётся не больше 4 файлов кода.

- [ ] **Step 5: Commit**

```bash
git add server/src/store
git commit -m "Add SQLite page and revision repository"
```

---

### Task 8: state, сессии и `Actor` из запроса

**Files:**
- Create: `server/src/http/mod.rs`, `state.rs`, `middleware.rs`, `rate_limit.rs`
- Modify: `server/src/lib.rs`, `server/src/main.rs`, `server/src/auth/tokens/mod.rs`; Create: `server/src/auth/tokens/session.rs`
- Create: `server/src/store/access/mod.rs`, `users.rs`, `keys.rs`, `proposals.rs`

**Interfaces:**
- Produces:
```rust
// http/state.rs
#[derive(Clone)]
pub struct AppState {
    pub pages: Arc<dyn PageRepo>,
    pub access: Arc<dyn AccessRepo>,
    pub proposals: Arc<dyn ProposalRepo>,
    pub sessions: Arc<Sessions>,
    pub limiter: Arc<RateLimiter>,
}
// auth/tokens/session.rs
pub struct Sessions;  // new(), create(user_id)->String, user_id(token:&str)->Option<i64>, destroy(token)
// http/rate_limit.rs
pub struct RateLimiter;  // new(max: u32, per: Duration), check(key:&str)->Result<(), u64 /*retry secs*/>
// http/middleware.rs
pub struct Auth(pub Actor);  // axum extractor: Bearer ключ, затем cookie "sid"
// store/access: SqliteAccess, SqliteProposals — реализации портов
```
Сессии в памяти (`Mutex<HashMap<String, i64>>`): при перезапуске сервера админов просят войти заново. Это сознательно: таблица сессий не нужна.

- [ ] **Step 1: Падающие тесты**

В `access/mod.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ports::access::AccessRepo;
    use crate::store::db::open_memory;

    #[test]
    fn upsert_user_is_stable_and_updates_login() {
        let a = SqliteAccess::new(open_memory().unwrap());
        let u1 = a.upsert_user(42, "old").unwrap();
        let u2 = a.upsert_user(42, "new").unwrap();
        assert_eq!(u1.id, u2.id);
        assert_eq!(u2.login, "new");
    }

    #[test]
    fn keys_found_by_hash_until_revoked() {
        let a = SqliteAccess::new(open_memory().unwrap());
        let id = a.create_key("ci", "h1", true, true, false).unwrap();
        let k = a.key_by_hash("h1").unwrap().unwrap();
        assert!(k.write && !k.export);
        a.revoke_key(id).unwrap();
        assert!(a.key_by_hash("h1").unwrap().is_none());
    }

    #[test]
    fn block_and_admin_flags() {
        let a = SqliteAccess::new(open_memory().unwrap());
        let u = a.upsert_user(1, "x").unwrap();
        a.set_blocked(u.id, true).unwrap();
        a.set_admin(u.id, true).unwrap();
        let u = a.user(u.id).unwrap().unwrap();
        assert!(u.blocked && u.admin);
    }
}
```

В `session.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_maps_to_user_until_destroyed() {
        let s = Sessions::new();
        let t = s.create(7);
        assert_eq!(s.user_id(&t), Some(7));
        s.destroy(&t);
        assert_eq!(s.user_id(&t), None);
        assert_eq!(s.user_id("nope"), None);
    }
}
```

В `rate_limit.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn blocks_after_max_and_reports_retry() {
        let l = RateLimiter::new(2, Duration::from_secs(60));
        assert!(l.check("ip").is_ok());
        assert!(l.check("ip").is_ok());
        let retry = l.check("ip").unwrap_err();
        assert!((1..=60).contains(&retry));
        assert!(l.check("other").is_ok());
    }
}
```

- [ ] **Step 2: Запустить** — `cargo test -p oc2r-wiki access:: session:: rate_limit::` → FAIL.

- [ ] **Step 3: Реализовать**

`auth/tokens/session.rs` (над тестами):
```rust
use rand::RngCore;
use std::{collections::HashMap, sync::Mutex};

#[derive(Default)]
pub struct Sessions {
    map: Mutex<HashMap<String, i64>>,
}

impl Sessions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create(&self, user_id: i64) -> String {
        let mut raw = [0u8; 24];
        rand::rng().fill_bytes(&mut raw);
        let token: String = raw.iter().map(|b| format!("{b:02x}")).collect();
        self.map.lock().unwrap().insert(token.clone(), user_id);
        token
    }

    pub fn user_id(&self, token: &str) -> Option<i64> {
        self.map.lock().unwrap().get(token).copied()
    }

    pub fn destroy(&self, token: &str) {
        self.map.lock().unwrap().remove(token);
    }
}
```
В `auth/tokens/mod.rs` добавить `pub mod session;`.

`http/rate_limit.rs` (над тестами):
```rust
use std::{collections::HashMap, sync::Mutex, time::{Duration, Instant}};

pub struct RateLimiter {
    max: u32,
    per: Duration,
    hits: Mutex<HashMap<String, (Instant, u32)>>,
}

impl RateLimiter {
    pub fn new(max: u32, per: Duration) -> Self {
        Self { max, per, hits: Mutex::new(HashMap::new()) }
    }

    /// Ok, если лимит не исчерпан; Err — через сколько секунд повторить.
    pub fn check(&self, key: &str) -> Result<(), u64> {
        let mut hits = self.hits.lock().unwrap();
        let now = Instant::now();
        let entry = hits.entry(key.to_owned()).or_insert((now, 0));
        if now.duration_since(entry.0) >= self.per {
            *entry = (now, 0);
        }
        if entry.1 >= self.max {
            return Err((self.per - now.duration_since(entry.0)).as_secs().max(1));
        }
        entry.1 += 1;
        Ok(())
    }
}
```

`store/access/users.rs`, `keys.rs`, `proposals.rs`: SQL по схеме из Task 6. Пример `keys.rs`:
```rust
use crate::domain::ports::{StoreResult, access::KeyRow};
use crate::store::db::{err, now};
use rusqlite::{Connection, OptionalExtension, params};

pub fn create(conn: &Connection, name: &str, hash: &str, r: bool, w: bool, e: bool) -> StoreResult<i64> {
    conn.execute(
        "insert into api_keys (name, key_hash, can_read, can_write, can_export, created_at) values (?1,?2,?3,?4,?5,?6)",
        params![name, hash, r, w, e, now()],
    )
    .map_err(err)?;
    Ok(conn.last_insert_rowid())
}

fn row(r: &rusqlite::Row) -> rusqlite::Result<KeyRow> {
    Ok(KeyRow { id: r.get(0)?, name: r.get(1)?, read: r.get(2)?, write: r.get(3)?, export: r.get(4)? })
}

pub fn by_hash(conn: &Connection, hash: &str) -> StoreResult<Option<KeyRow>> {
    conn.query_row(
        "select id, name, can_read, can_write, can_export from api_keys where key_hash = ?1 and revoked_at is null",
        [hash],
        row,
    )
    .optional()
    .map_err(err)
}

pub fn touch(conn: &Connection, id: i64) -> StoreResult<()> {
    conn.execute("update api_keys set last_used_at = ?1 where id = ?2", params![now(), id]).map_err(err)?;
    Ok(())
}

pub fn revoke(conn: &Connection, id: i64) -> StoreResult<()> {
    conn.execute("update api_keys set revoked_at = ?1 where id = ?2", params![now(), id]).map_err(err)?;
    Ok(())
}

pub fn list(conn: &Connection) -> StoreResult<Vec<KeyRow>> {
    let mut stmt = conn
        .prepare("select id, name, can_read, can_write, can_export from api_keys where revoked_at is null order by id")
        .map_err(err)?;
    stmt.query_map([], row).map_err(err)?.collect::<Result<_, _>>().map_err(err)
}
```

`users.rs` — `upsert` (`insert ... on conflict(github_id) do update set login = excluded.login returning id, github_id, login, admin, blocked_at is not null`), `get`, `set_blocked` (`blocked_at = now() или null`), `set_admin`.

`proposals.rs` — `create`, `get`, `list(status)`, `count_open_by_user`, `count_since_by_user`, `decide` (устанавливает `status`, `decided_by`, `decided_at`, `decision_note`).

`access/mod.rs` (над тестами): структуры `SqliteAccess { db }` и `SqliteProposals { db }` с `new(db)`, реализующие `AccessRepo` и `ProposalRepo` вызовами функций из `users.rs`, `keys.rs`, `proposals.rs`. Папка содержит ровно 4 файла.

`http/state.rs`:
```rust
use crate::auth::tokens::session::Sessions;
use crate::domain::ports::{access::AccessRepo, pages::PageRepo, proposals::ProposalRepo};
use crate::http::rate_limit::RateLimiter;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub pages: Arc<dyn PageRepo>,
    pub access: Arc<dyn AccessRepo>,
    pub proposals: Arc<dyn ProposalRepo>,
    pub sessions: Arc<Sessions>,
    pub limiter: Arc<RateLimiter>,
}
```

`http/middleware.rs` — экстрактор `Auth`:
```rust
use super::state::AppState;
use crate::auth::{policy::Actor, tokens::api_key};
use axum::{extract::FromRequestParts, http::request::Parts};
use std::convert::Infallible;

pub struct Auth(pub Actor);

fn cookie<'a>(parts: &'a Parts, name: &str) -> Option<&'a str> {
    let header = parts.headers.get("cookie")?.to_str().ok()?;
    header.split(';').filter_map(|p| p.trim().split_once('=')).find(|(k, _)| *k == name).map(|(_, v)| v)
}

impl FromRequestParts<AppState> for Auth {
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Infallible> {
        if let Some(key) = parts.headers.get("authorization").and_then(|v| v.to_str().ok()).and_then(|v| v.strip_prefix("Bearer ")) {
            return Ok(Auth(match state.access.key_by_hash(&api_key::hash(key)) {
                Ok(Some(k)) => {
                    let _ = state.access.touch_key(k.id);
                    Actor::Key { id: k.id, read: k.read, write: k.write, export: k.export }
                }
                _ => Actor::Anonymous,
            }));
        }
        let user = cookie(parts, "sid").and_then(|t| state.sessions.user_id(t)).and_then(|id| state.access.user(id).ok().flatten());
        Ok(Auth(match user {
            Some(u) => Actor::User { id: u.id, admin: u.admin, blocked: u.blocked },
            None => Actor::Anonymous,
        }))
    }
}
```
Неверный ключ превращается в `Anonymous`: обработчик, которому нужны права, сам вернёт 401 или 403 через `policy::can`.

`http/mod.rs`:
```rust
pub mod middleware;
pub mod rate_limit;
pub mod state;
```

`lib.rs`: `router(state: AppState) -> Router` (`.with_state(state)`), в тесте health собирается состояние из `open_memory()`. Вынести сборку состояния в `main.rs`: открыть БД по `Config.db_path`, создать `SqlitePages`, `SqliteAccess`, `SqliteProposals`.

- [ ] **Step 4: Запустить** — `cargo test -p oc2r-wiki` → PASS.

- [ ] **Step 5: Commit**

```bash
git add server
git commit -m "Add app state, sessions, rate limiter and access repositories"
```

---

### Task 9: API (`/api/...`)

**Files:**
- Create: `server/src/http/api/mod.rs`, `pages.rs`, `export.rs`
- Create: `server/tests/support/mod.rs` (общий помощник `app()` и `json(res)`), `server/tests/public/main.rs` (`#[path = "../support/mod.rs"] mod support; mod api; mod reader;`), `server/tests/public/api.rs`. Аналогично `server/tests/staff/main.rs` в задачах 11 и 12. Тестовых файлов в каждой папке не больше 4.
- Modify: `server/src/http/mod.rs`, `server/src/lib.rs`, `server/Cargo.toml` (добавить `zip`? нет: экспорт — один `.tar`-подобный JSON, см. ниже)

**Interfaces:**
- Consumes: `AppState`, `Auth`, `policy::can`, `PageRepo`, `AppError`.
- Produces: `api::routes() -> Router<AppState>`; маршруты из `docs/workflows.md`.

Экспорт без новых библиотек: `GET /api/export` отдаёт JSON `{ "pages": [{ "slug", "lang", "body" }] }`. Формат архива можно сменить позже, не трогая остальное.

- [ ] **Step 1: Падающие интеграционные тесты** (`server/tests/public/api.rs`)

Хелпер `fn app() -> (Router, String /*ключ write*/ , String /*ключ read*/)`: строит `AppState` на `open_memory()`, создаёт два ключа через `AccessRepo::create_key` с `api_key::generate()`. Тесты:
```rust
#[tokio::test]
async fn put_then_get_roundtrip() {
    let (app, write, _) = app();
    let put = Request::put("/api/pages/blocks/computer?lang=en&summary=init")
        .header("authorization", format!("Bearer {write}"))
        .body(Body::from("# Computer")).unwrap();
    assert_eq!(app.clone().oneshot(put).await.unwrap().status(), 200);
    let get = Request::get("/api/pages/blocks/computer?lang=en").body(Body::empty()).unwrap();
    let res = app.oneshot(get).await.unwrap();
    assert_eq!(res.status(), 200);
    assert_eq!(json(res).await["body"], "# Computer");
}

#[tokio::test]
async fn write_without_key_is_401_and_read_key_is_403() {
    let (app, _, read) = app();
    let anon = Request::put("/api/pages/a?lang=en").body(Body::from("x")).unwrap();
    assert_eq!(app.clone().oneshot(anon).await.unwrap().status(), 401);
    let ro = Request::put("/api/pages/a?lang=en").header("authorization", format!("Bearer {read}")).body(Body::from("x")).unwrap();
    assert_eq!(app.oneshot(ro).await.unwrap().status(), 403);
}

#[tokio::test]
async fn repeated_put_does_not_add_revision() {
    // PUT одним и тем же телом дважды; GET /api/pages/a/revisions?lang=en возвращает 1 элемент
}

#[tokio::test]
async fn bad_slug_or_lang_is_422() {
    // PUT /api/pages/Bad%20Slug?lang=en и PUT /api/pages/a?lang=de дают 422
}

#[tokio::test]
async fn export_needs_export_scope() {
    // read-ключ получает 403; ключ с export=true получает 200 и список страниц
}
```
(`json(res)` — помощник: `http_body_util::BodyExt::collect` + `serde_json::from_slice`.)

- [ ] **Step 2: Запустить** — `cargo test -p oc2r-wiki --test public` → FAIL.

- [ ] **Step 3: Реализовать**

`http/api/pages.rs`:
```rust
use crate::auth::policy::{Action, Actor, can};
use crate::domain::{page::{Lang, Slug}, revision::{Author, NewRevision}};
use crate::error::{AppError, AppResult};
use crate::http::{middleware::Auth, state::AppState};
use axum::{Json, extract::{Path, Query, State}};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize)]
pub struct PageQuery {
    pub lang: String,
    pub summary: Option<String>,
}

fn parse(slug: &str, lang: &str) -> AppResult<(Slug, Lang)> {
    let slug = Slug::parse(slug).ok_or_else(|| AppError::Invalid("slug".into()))?;
    let lang = Lang::parse(lang).ok_or_else(|| AppError::Invalid("lang".into()))?;
    Ok((slug, lang))
}

fn require(actor: &Actor, action: Action) -> AppResult<()> {
    if can(actor, action) {
        Ok(())
    } else if matches!(actor, Actor::Anonymous) {
        Err(AppError::Unauthorized)
    } else {
        Err(AppError::Forbidden)
    }
}

pub fn author(actor: &Actor) -> Author {
    match actor {
        Actor::Key { id, .. } => Author::ApiKey(format!("#{id}")),
        Actor::User { id, .. } => Author::User(format!("#{id}")),
        Actor::Anonymous => Author::Import,
    }
}

pub async fn get_page(State(s): State<AppState>, Path(slug): Path<String>, Query(q): Query<PageQuery>) -> AppResult<Json<Value>> {
    let (slug, lang) = parse(&slug, &q.lang)?;
    let page = s.pages.find(&slug, lang)?.ok_or(AppError::NotFound)?;
    Ok(Json(json!({ "slug": slug.as_str(), "lang": lang.code(), "body": page.current.body, "revision": page.current.id })))
}

pub async fn put_page(State(s): State<AppState>, Auth(actor): Auth, Path(slug): Path<String>, Query(q): Query<PageQuery>, body: String) -> AppResult<Json<Value>> {
    require(&actor, Action::WritePage)?;
    let (slug, lang) = parse(&slug, &q.lang)?;
    let rev = NewRevision { body, summary: q.summary.unwrap_or_default(), author: author(&actor) };
    let outcome = s.pages.save(&slug, lang, rev)?;
    Ok(Json(json!({ "outcome": format!("{outcome:?}") })))
}

pub async fn revisions(State(s): State<AppState>, Path(slug): Path<String>, Query(q): Query<PageQuery>) -> AppResult<Json<Value>> {
    let (slug, lang) = parse(&slug, &q.lang)?;
    let page = s.pages.find(&slug, lang)?.ok_or(AppError::NotFound)?;
    let items: Vec<Value> = s.pages.history(page.id)?.iter()
        .map(|r| json!({ "id": r.id, "summary": r.summary, "author": r.author.label(), "created_at": r.created_at }))
        .collect();
    Ok(Json(json!(items)))
}

pub async fn list(State(s): State<AppState>) -> AppResult<Json<Value>> {
    let items: Vec<Value> = s.pages.list()?.iter()
        .map(|p| json!({ "slug": p.slug.as_str(), "lang": p.lang.code(), "section": p.section }))
        .collect();
    Ok(Json(json!(items)))
}
```
Маршрут со слэшами в slug: `/api/pages/{*slug}`. Путь `/revisions` для вложенных slug разбирается как суффикс (в `mod.rs`: отдельный маршрут `/api/revisions/{*slug}`, чтобы избежать неоднозначности; в `docs/workflows.md` поправить таблицу тем же шагом).

`http/api/export.rs`:
```rust
use super::pages::require;
use crate::auth::policy::Action;
use crate::error::AppResult;
use crate::http::{middleware::Auth, state::AppState};
use axum::{Json, extract::State};
use serde_json::{Value, json};

pub async fn export(State(s): State<AppState>, Auth(actor): Auth) -> AppResult<Json<Value>> {
    require(&actor, Action::Export)?;
    let mut pages = Vec::new();
    for p in s.pages.list()? {
        if let Some(page) = s.pages.find(&p.slug, p.lang)? {
            pages.push(json!({ "slug": p.slug.as_str(), "lang": p.lang.code(), "body": page.current.body }));
        }
    }
    Ok(Json(json!({ "pages": pages })))
}
```
(`require` в `pages.rs` объявить `pub`.)

`http/api/mod.rs`:
```rust
mod export;
mod pages;

use super::state::AppState;
use axum::{Router, routing::get};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/pages", get(pages::list))
        .route("/api/pages/{*slug}", get(pages::get_page).put(pages::put_page))
        .route("/api/revisions/{*slug}", get(pages::revisions))
        .route("/api/export", get(export::export))
}
```

`lib.rs`: `Router::new().route("/health", ...).merge(http::api::routes()).with_state(state)`.

- [ ] **Step 4: Запустить** — `cargo test -p oc2r-wiki` → PASS.

- [ ] **Step 5: Commit**

```bash
git add server
git commit -m "Add key-authenticated page API and export"
```

---

### Task 10: страницы читателя (SSR)

**Files:**
- Create: `server/src/http/reader/mod.rs`, `articles.rs`, `history.rs`, `search.rs`
- Create: `server/templates/layout/base.html`, `server/templates/reader/article.html`, `history.html`, `search.html`
- Modify: `server/Cargo.toml` (добавить `askama = "0.14"`), `server/src/http/mod.rs`, `server/src/lib.rs`

**Interfaces:**
- Consumes: `PageRepo`, `render::to_html`, `diff::diff_lines`, `size_delta`.
- Produces маршруты: `GET /{lang}/{*slug}` (статья; `?rev=N` — старая версия; `?history` — список; `?diff=A..B` — сравнение), `GET /{lang}/search?q=`, `GET /` (редирект на `/en/start`).
- Шаблоны получают `lang`, `title`, `html`, `old_rev: Option<OldRev>`, `missing_translation: bool`.

- [ ] **Step 1: Падающие тесты** (`server/tests/public/reader.rs`)

```rust
#[tokio::test]
async fn article_renders_markdown() {
    // сохранить "# Hello" в a/en, GET /en/a → 200, тело содержит "<h1>Hello</h1>"
}

#[tokio::test]
async fn old_revision_shows_banner_and_old_text() {
    // две ревизии "one", "two"; GET /en/a?rev=<id первой> → содержит "one" и "Старая версия", не содержит "two"
}

#[tokio::test]
async fn missing_translation_falls_back_to_english_with_notice() {
    // сохранить только en; GET /ru/a → 200, содержит текст en и "Перевода пока нет"
}

#[tokio::test]
async fn unknown_page_is_404_with_search_box() {
    // GET /en/nope → 404, тело содержит <form с action поиска
}

#[tokio::test]
async fn history_lists_revisions_and_diff_marks_changes() {
    // GET /en/a?history перечисляет обе ревизии; GET /en/a?diff=1..2 содержит <del> и <ins>
}

#[tokio::test]
async fn deleted_page_is_hidden() {
    // set_deleted(true) → GET /en/a → 404
}
```

- [ ] **Step 2: Запустить** — `cargo test -p oc2r-wiki --test public` → FAIL.

- [ ] **Step 3: Реализовать**

`articles.rs`: один обработчик читает `Query<ArticleQuery { rev: Option<i64>, history: Option<String>, diff: Option<String> }>` и выбирает шаблон: `history` → `history::render`, `diff` → `history::render_diff`, иначе статья. Логика выбора версии:
```rust
let page = match s.pages.find(&slug, lang)? {
    Some(p) => (p, false),
    None if lang != Lang::En => (s.pages.find(&slug, Lang::En)?.ok_or(AppError::NotFound)?, true),
    None => return Err(AppError::NotFound),
};
let (page, missing_translation) = page;
let (revision, old) = match q.rev {
    Some(id) => (s.pages.revision(page.id, id)?.ok_or(AppError::NotFound)?, true),
    None => (page.current.clone(), false),
};
let html = render::to_html(&revision.body);
```
Ошибка `NotFound` в читательской части рендерится как HTML-страница 404 с формой поиска (отдельная функция `not_found_page`), а в `/api` остаётся JSON. Различие делается по префиксу пути в обработчике, не в `AppError`.

`history.rs`: список ревизий (`size_delta` между соседними), `diff_lines` → `<ins>`, `<del>`; текст экранируется функцией `escape` (askama делает это сам для `{{ }}`).

Шаблон `layout/base.html`: `<head>` с inline-скриптом темы (читает `localStorage`, ставит `data-theme`, `data-accent`, `data-pixel`, `data-size` на `<html>` до отрисовки, всё в `try/catch`), подключение `/static/app.css` и `/static/app.js`, шапка (логотип, форма поиска `action="/{{ lang }}/search"`, переключатель языка, кнопка ⚙), блок `content`.

Шаблон `reader/article.html`: `{% extends "layout/base.html" %}`, плашка старой версии (`old_rev`), плашка отсутствующего перевода, `{{ html|safe }}`, ссылки «История» и «Предложить правку».

- [ ] **Step 4: Запустить** — `cargo test -p oc2r-wiki` → PASS.

- [ ] **Step 5: Commit**

```bash
git add server
git commit -m "Add reader pages with history, old revisions and diff"
```

---

### Task 11: предложения правок и вход через GitHub

**Files:**
- Create: `server/src/http/user/mod.rs`, `login.rs`, `propose.rs`
- Create: `server/src/auth/github.rs`
- Create: `server/templates/reader/propose.html`
- Modify: `server/Cargo.toml` (добавить `reqwest = { version = "0.12", default-features = false, features = ["rustls-tls", "json"] }`), `server/src/config.rs` (поля `github_client_id`, `github_client_secret`, `public_url`, `admin_logins: Vec<String>`), `server/src/auth/mod.rs`

**Interfaces:**
- Produces:
```rust
// auth/github.rs
pub fn authorize_url(client_id: &str, redirect: &str, state: &str) -> String
pub async fn exchange(cfg: &Config, code: &str) -> Result<GithubUser, String>   // GithubUser { id: i64, login: String }
// http/user/propose.rs
pub const MAX_OPEN: u32 = 5;
pub const MAX_PER_DAY: u32 = 20;
pub fn check_quota(open: u32, today: u32) -> Result<(), AppError>
```
Маршруты: `GET /auth/github` (редирект на GitHub, `state` в cookie), `GET /auth/github/callback` (обмен кода, `upsert_user`, админ по `admin_logins`, cookie `sid` с `HttpOnly; SameSite=Lax; Secure; Path=/`), `POST /auth/logout`, `GET/POST /{lang}/{*slug}/propose`.

- [ ] **Step 1: Падающие тесты**

В `propose.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quota_limits() {
        assert!(check_quota(0, 0).is_ok());
        assert!(matches!(check_quota(MAX_OPEN, 0), Err(AppError::RateLimited(_))));
        assert!(matches!(check_quota(0, MAX_PER_DAY), Err(AppError::RateLimited(_))));
    }
}
```

В `github.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authorize_url_has_client_state_and_encoded_redirect() {
        let u = authorize_url("cid", "https://wiki.example/auth/github/callback", "st");
        assert!(u.starts_with("https://github.com/login/oauth/authorize?"));
        assert!(u.contains("client_id=cid") && u.contains("state=st"));
        assert!(u.contains("redirect_uri=https%3A%2F%2Fwiki.example%2Fauth%2Fgithub%2Fcallback"));
    }
}
```

Интеграционные (`server/tests/staff/proposals.rs`), пользователь создаётся через `AccessRepo::upsert_user` и сессия через `Sessions::create`, cookie подставляется вручную:
```rust
#[tokio::test]
async fn anonymous_cannot_propose() { /* POST → 401 */ }

#[tokio::test]
async fn blocked_user_cannot_propose() { /* POST → 403 */ }

#[tokio::test]
async fn proposal_is_stored_with_base_revision() { /* POST → 303; proposals.list(Open) содержит base_rev_id = текущая */ }

#[tokio::test]
async fn sixth_open_proposal_is_rate_limited() { /* 5 успешных, шестой → 429 */ }
```

- [ ] **Step 2: Запустить** — `cargo test -p oc2r-wiki` → FAIL.

- [ ] **Step 3: Реализовать**

`github.rs` (над тестами):
```rust
use crate::config::Config;
use serde::Deserialize;

pub struct GithubUser {
    pub id: i64,
    pub login: String,
}

fn encode(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

pub fn authorize_url(client_id: &str, redirect: &str, state: &str) -> String {
    format!(
        "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}&state={}",
        encode(client_id), encode(redirect), encode(state)
    )
}

#[derive(Deserialize)]
struct Token {
    access_token: String,
}

#[derive(Deserialize)]
struct Me {
    id: i64,
    login: String,
}

pub async fn exchange(cfg: &Config, code: &str) -> Result<GithubUser, String> {
    let http = reqwest::Client::new();
    let token: Token = http
        .post("https://github.com/login/oauth/access_token")
        .header("accept", "application/json")
        .form(&[("client_id", &cfg.github_client_id), ("client_secret", &cfg.github_client_secret), ("code", &code.to_owned())])
        .send().await.map_err(|e| e.to_string())?
        .json().await.map_err(|e| e.to_string())?;
    let me: Me = http
        .get("https://api.github.com/user")
        .header("user-agent", "oc2r-wiki")
        .bearer_auth(token.access_token)
        .send().await.map_err(|e| e.to_string())?
        .json().await.map_err(|e| e.to_string())?;
    Ok(GithubUser { id: me.id, login: me.login })
}
```

`propose.rs`: обработчик `POST` — `Auth`, `require(Action::Propose)`, `check_quota(count_open_by_user, count_since_by_user(now - 86400))`, `proposals.create(page.id, page.current.id, body, summary, user_id)`, редирект `303` на статью. `GET` показывает форму с текущим текстом.

`login.rs`: callback проверяет `state` из cookie и параметра, вызывает `github::exchange`, `upsert_user`, при совпадении `login` с `admin_logins` — `set_admin(id, true)`, создаёт сессию. При ошибке GitHub возвращает страницу с сообщением и ссылкой «Войти снова» (статус 502, не 500).

- [ ] **Step 4: Запустить** — `cargo test -p oc2r-wiki` → PASS (OAuth-обмен сети не касается: он не тестируется автоматически, ручная проверка в Task 14).

- [ ] **Step 5: Commit**

```bash
git add server
git commit -m "Add GitHub login and proposal submission with quotas"
```

---

### Task 12: админка

**Files:**
- Create: `server/src/http/admin/mod.rs`, `pages.rs`, `proposals.rs`, `keys.rs`
- Create: `server/templates/admin/dashboard.html`, `proposals.html`, `keys.html`, `editor.html`
- Modify: `server/src/http/mod.rs`, `server/src/lib.rs`

**Interfaces:**
- Produces (все маршруты требуют `policy::can`, иначе 401 или 403):
  - `GET /admin` — список страниц и ссылки на разделы.
  - `GET|POST /admin/edit/{lang}/{*slug}` — редактор; POST сохраняет через `PageRepo::save` с `Author::User(login)`.
  - `POST /admin/rollback/{lang}/{*slug}` с полем `rev` — `rollback_to` + `save`.
  - `POST /admin/delete/{lang}/{*slug}` и `/admin/restore/...` — `set_deleted`.
  - `GET /admin/proposals`, `GET /admin/proposals/{id}` (diff против базовой и текущей ревизий; при `is_conflict` показывается трёхстороннее сравнение), `POST /admin/proposals/{id}/accept` и `/reject`.
  - `GET /admin/keys`, `POST /admin/keys` (создать: показывает ключ один раз), `POST /admin/keys/{id}/revoke`.
  - `POST /admin/preview` — тело Markdown → HTML через `render::to_html` (для редактора).

- [ ] **Step 1: Падающие тесты** (`server/tests/staff/admin.rs`)

```rust
#[tokio::test]
async fn non_admin_is_403_and_anonymous_is_401() { /* GET /admin */ }

#[tokio::test]
async fn admin_edit_creates_revision_authored_by_admin() { /* POST /admin/edit/en/a; history автор = логин админа */ }

#[tokio::test]
async fn rollback_creates_new_revision_not_rewrite() { /* история растёт на 1; старые ревизии на месте */ }

#[tokio::test]
async fn accept_clean_proposal_publishes_with_proposer_as_author() { /* base == current → accepted, новая ревизия */ }

#[tokio::test]
async fn accept_conflicting_proposal_requires_manual_merge() { /* base != current: POST accept без поля body → 422; с итоговым body → ok */ }

#[tokio::test]
async fn reject_stores_note_and_changes_nothing() { /* статус rejected, страница не изменилась */ }

#[tokio::test]
async fn key_shown_once_and_revoked_key_stops_working() {
    // POST /admin/keys → ответ содержит ключ ocw_...; повторный GET /admin/keys не содержит; после revoke PUT /api/pages → 401
}

#[tokio::test]
async fn preview_uses_the_shared_renderer() { /* POST /admin/preview "**x**" → "<strong>x</strong>" */ }
```

- [ ] **Step 2: Запустить** — `cargo test -p oc2r-wiki --test staff` → FAIL.

- [ ] **Step 3: Реализовать**

Общий помощник в `admin/mod.rs`:
```rust
pub fn admin_only(actor: &Actor, action: Action) -> AppResult<()> {
    crate::http::api::pages::require(actor, action)
}
```
(`require` остаётся единственным местом перевода «нет прав» в 401/403; поднять его в `http/mod.rs` как `pub fn require`, чтобы `api` и `admin` не зависели друг от друга. Тот же шаг: перенести объявление и поправить импорты Task 9.)

Принятие предложения:
```rust
let p = s.proposals.get(id)?.ok_or(AppError::NotFound)?;
let page = s.pages.find_by_id(p.page_id)?;           // см. ниже
let body = if is_conflict(p.base_rev_id, page.current.id) {
    form.body.ok_or_else(|| AppError::Invalid("Конфликт: пришлите итоговый текст".into()))?
} else {
    p.body.clone()
};
s.pages.save(&page.slug, page.lang, NewRevision {
    body,
    summary: format!("{} (принял {})", p.summary, admin_login),
    author: Author::User(proposer_login),
})?;
s.proposals.decide(id, Status::Accepted, admin_id, &form.note)?;
```
Для этого в `PageRepo` нужен `find_by_id(page_id) -> StoreResult<Option<Page>>`: добавить в порт и в `SqlitePages` (тест: `find_by_id` возвращает ту же страницу, что и `find`), коммит отдельным шагом до остальных изменений задачи.

Шаблон редактора: `<textarea>`, кнопки вставки (подключает `editor.js`), область превью. Без JS форма работает как обычный POST.

- [ ] **Step 4: Запустить** — `cargo test -p oc2r-wiki` → PASS.

- [ ] **Step 5: Commit**

```bash
git add server
git commit -m "Add admin panel for pages, proposals and API keys"
```

---

### Task 13: веб-ресурсы (стили, настройки, поиск, редактор)

**Files:**
- Create: `web/package.json`, `web/tsconfig.json`, `web/build.ts`
- Create: `web/src/main.ts`
- Create: `web/src/settings/store.ts`, `theme.ts`, `panel.ts`
- Create: `web/src/search/index.ts`, `ui.ts`
- Create: `web/src/editor/editor.ts`, `preview.ts`, `toolbar.ts`, `diffview.ts`
- Create: `web/src/shared/dom.ts`, `http.ts`, `keys.ts`
- Create: `web/styles/tokens.css`, `layout.css`, `components.css`, `pixel.css`
- Modify: `server/Cargo.toml` (`rust-embed = "8"`), `server/src/lib.rs` (маршрут `/static/{*path}` из встроенной папки `web/dist`)

**Interfaces:**
- `settings/store.ts`:
```ts
export type Theme = "light" | "dark" | "auto";
export type Accent = "green" | "violet" | "orange" | "teal";
export type Size = "s" | "m" | "l";
export interface Settings { theme: Theme; accent: Accent; size: Size; pixel: boolean }
export const DEFAULTS: Settings = { theme: "auto", accent: "green", size: "m", pixel: true };
export function load(storage: Pick<Storage, "getItem">): Settings;
export function save(storage: Pick<Storage, "setItem">, s: Settings): void;
```
- `settings/theme.ts`: `export function apply(root: HTMLElement, s: Settings, prefersDark: boolean): void` — ставит `data-theme` (`light`/`dark`), `data-accent`, `data-size`, `data-pixel` ("1"/"0").

- [ ] **Step 1: Падающий тест настроек** (`web/src/settings/store.test.ts`, запуск через `bun test`, встроенный раннер, без тестовых библиотек)

```ts
import { expect, test } from "bun:test";
import { DEFAULTS, load, save } from "./store.ts";

const fake = (v: string | null) => ({ getItem: () => v });

test("returns defaults when storage is empty or broken", () => {
  expect(load(fake(null))).toEqual(DEFAULTS);
  expect(load(fake("{not json"))).toEqual(DEFAULTS);
  expect(load({ getItem: () => { throw new Error("blocked"); } })).toEqual(DEFAULTS);
});

test("ignores unknown values and keeps valid ones", () => {
  const s = load(fake(JSON.stringify({ theme: "dark", accent: "pink", size: "l", pixel: false })));
  expect(s.theme).toBe("dark");
  expect(s.accent).toBe(DEFAULTS.accent);
  expect(s.size).toBe("l");
  expect(s.pixel).toBe(false);
});

test("save swallows storage errors", () => {
  expect(() => save({ setItem: () => { throw new Error("full"); } }, DEFAULTS)).not.toThrow();
});
```

- [ ] **Step 2: Запустить** — `cd web && bun test src/settings/store.test.ts` → FAIL (модуля нет).

- [ ] **Step 3: Реализовать**

`web/package.json`:
```json
{
  "name": "oc2r-wiki-web",
  "private": true,
  "license": "GPL-3.0-or-later",
  "type": "module",
  "scripts": {
    "build": "bun run build.ts",
    "test": "bun test",
    "check": "tsc --noEmit"
  },
  "devDependencies": { "@types/bun": "latest", "typescript": "^5.8.0" }
}
```

`web/src/settings/store.ts`:
```ts
export type Theme = "light" | "dark" | "auto";
export type Accent = "green" | "violet" | "orange" | "teal";
export type Size = "s" | "m" | "l";
export interface Settings { theme: Theme; accent: Accent; size: Size; pixel: boolean }

export const DEFAULTS: Settings = { theme: "auto", accent: "green", size: "m", pixel: true };
const KEY = "oc2r-wiki-settings";

const THEMES: Theme[] = ["light", "dark", "auto"];
const ACCENTS: Accent[] = ["green", "violet", "orange", "teal"];
const SIZES: Size[] = ["s", "m", "l"];

function pick<T>(allowed: T[], value: unknown, fallback: T): T {
  return allowed.includes(value as T) ? (value as T) : fallback;
}

export function load(storage: Pick<Storage, "getItem">): Settings {
  try {
    const raw = storage.getItem(KEY);
    if (!raw) return { ...DEFAULTS };
    const v = JSON.parse(raw) as Record<string, unknown>;
    return {
      theme: pick(THEMES, v.theme, DEFAULTS.theme),
      accent: pick(ACCENTS, v.accent, DEFAULTS.accent),
      size: pick(SIZES, v.size, DEFAULTS.size),
      pixel: typeof v.pixel === "boolean" ? v.pixel : DEFAULTS.pixel,
    };
  } catch {
    return { ...DEFAULTS };
  }
}

export function save(storage: Pick<Storage, "setItem">, s: Settings): void {
  try {
    storage.setItem(KEY, JSON.stringify(s));
  } catch {
    /* хранилище недоступно: сайт работает с настройками по умолчанию */
  }
}
```

`web/src/settings/theme.ts`:
```ts
import type { Settings } from "./store.ts";

export function apply(root: HTMLElement, s: Settings, prefersDark: boolean): void {
  root.dataset.theme = s.theme === "auto" ? (prefersDark ? "dark" : "light") : s.theme;
  root.dataset.accent = s.accent;
  root.dataset.size = s.size;
  root.dataset.pixel = s.pixel ? "1" : "0";
}
```

`web/build.ts`:
```ts
const results = await Promise.all([
  Bun.build({ entrypoints: ["src/main.ts"], outdir: "dist", target: "browser", minify: true, naming: "app.[ext]" }),
  Bun.build({ entrypoints: ["styles/tokens.css"], outdir: "dist", minify: true, naming: "app.[ext]" }),
]);

for (const r of results) {
  if (!r.success) {
    console.error(r.logs);
    process.exit(1);
  }
}
```
Две отдельные сборки нужны, чтобы JS и CSS получили имена `app.js` и `app.css`. `tokens.css` импортирует остальные три файла через `@import`, Bun склеивает их в один `dist/app.css`; проверить, что в нём есть правила всех четырёх файлов.

`web/tsconfig.json`: `"strict": true`, `"target": "ES2022"`, `"module": "ESNext"`, `"moduleResolution": "bundler"`, `"lib": ["ES2022", "DOM"]`, `"types": ["bun"]`, `"allowImportingTsExtensions": true`, `"noEmit": true`.

`styles/tokens.css`: переменные `--bg`, `--fg`, `--muted`, `--line`, `--accent` (по `[data-accent]`), `--font-size` (по `[data-size]`), тёмные значения под `[data-theme="dark"]`, `--pixel-border: 0` и `1px solid var(--line)`; при `[data-pixel="1"]`: `3px solid #1a1a1a` и жёсткая тень `3px 3px 0 #1a1a1a`. `pixel.css`: рамки шапки и сайдбара, плашки `.callout`, крафт-сетка `.recipe` (3×3, `.slot` с вдавленной рамкой). `layout.css`: сетка «сайдбар, статья, оглавление» (на узком экране в одну колонку). `components.css`: кнопки, поле поиска, панель настроек, diff (`ins`, `del`). Минимальный контраст AA проверить на обеих темах (Step 5).

`search/ui.ts`: клавиша `/` фокусирует поле поиска (если фокус не в поле ввода). `search/index.ts`: на ввод запрашивает `/{lang}/search?q=` как `fragment` (сервер отдаёт готовый HTML-список) и вставляет под полем; без JS форма идёт обычным GET.

`editor/*`: `toolbar.ts` вставляет `:::tip`, `:::warn`, `:::recipe`, `:::version` в позицию курсора; `preview.ts` с задержкой 300 мс шлёт текст на `POST /admin/preview` и вставляет ответ в область превью; `diffview.ts` подсвечивает `ins`/`del` для трёхстороннего сравнения; `editor.ts` подключает всё к `#editor`.

`main.ts`: применяет настройки, собирает панель (`panel.ts`), подключает поиск; редактор загружается только если на странице есть `#editor`.

- [ ] **Step 4: Запустить** — `cd web && bun install && bun test` → PASS; `bun run build` → создаёт `dist/app.js` и `dist/app.css`.

- [ ] **Step 5: Ручная проверка**

Run: `cargo run -p oc2r-wiki` и открыть `http://127.0.0.1:8080/en/a`.
Expected: тема переключается без вспышки; смена акцента, размера и пиксель-рамок применяется сразу и переживает перезагрузку; с отключённым JS статья читается; `/` фокусирует поиск; при блокировке `localStorage` (режим инкогнито с запретом данных) сайт открывается с настройками по умолчанию.

- [ ] **Step 6: Commit**

```bash
git add web server/Cargo.toml server/src/lib.rs
git commit -m "Add frontend settings, search, editor and styles"
```

---

### Task 14: деплой, README и сквозной тест

**Files:**
- Create: `deploy/Caddyfile`, `deploy/oc2r-wiki.service`, `deploy/backup.sh` (в продакшене вместо Caddy работает nginx: `deploy/nginx-wiki.conf`, unit в `deploy/systemd/`)
- Create: `README.md`
- Create: `server/tests/e2e.rs`

- [ ] **Step 1: Сквозной тест** (`server/tests/e2e.rs`)

Сценарий одним тестом на `open_memory()`:
1. Ключ с `write` заливает `a/en` («v1»), затем («v2»).
2. Вошедший пользователь отправляет предложение на основе ревизии «v2».
3. Админ принимает: на странице новый текст, автор ревизии — предложивший.
4. В истории 3 ревизии; `GET /en/a?rev=<id первой>` показывает «v1» с плашкой.
5. Админ откатывает к первой ревизии: в истории 4 записи, актуальный текст «v1».

```rust
#[tokio::test]
async fn propose_accept_history_rollback() { /* шаги 1–5; после каждого шага assert на тело/статус */ }
```

- [ ] **Step 2: Запустить** — `cargo test -p oc2r-wiki --test e2e` → PASS (если красный, чинить реализацию задач 7–12, а не тест).

- [ ] **Step 3: Деплой-файлы**

`deploy/Caddyfile` (исходный план; в продакшене используется nginx, см. `deploy/nginx-wiki.conf`):
```
wiki.loki-code.dev {
    encode zstd gzip
    reverse_proxy 127.0.0.1:8080
}
```

`deploy/oc2r-wiki.service`:
```ini
[Unit]
Description=OC2R Wiki
After=network.target

[Service]
User=wiki
WorkingDirectory=/var/lib/oc2r-wiki
EnvironmentFile=/etc/oc2r-wiki.env
ExecStart=/usr/local/bin/oc2r-wiki
Restart=on-failure
NoNewPrivileges=true
ProtectSystem=strict
ReadWritePaths=/var/lib/oc2r-wiki

[Install]
WantedBy=multi-user.target
```

`deploy/backup.sh`:
```sh
#!/bin/sh
# Копия базы раз в неделю (cron/systemd timer) и перед обновлением.
set -eu
DB="${WIKI_DB:-/var/lib/oc2r-wiki/wiki.db}"
OUT="${1:-/var/backups/oc2r-wiki}"
mkdir -p "$OUT"
sqlite3 "$DB" ".backup '$OUT/wiki-$(date +%F).db'"
find "$OUT" -name 'wiki-*.db' -mtime +60 -delete
```

`README.md`: что это, лицензия GPL-3.0-or-later, сборка (`cd web && bun install --frozen-lockfile && bun run build`, затем `cargo build --release -p oc2r-wiki`), переменные окружения (`WIKI_BIND`, `WIKI_DB`, `GITHUB_CLIENT_ID`, `GITHUB_CLIENT_SECRET`, `PUBLIC_URL`, `ADMIN_LOGINS`), как создать OAuth-приложение GitHub, как завести ключ API, пример `curl -X PUT -H "Authorization: Bearer $KEY" --data-binary @page.md "https://wiki.loki-code.dev/api/pages/start?lang=en&summary=sync"`, ссылка на `docs/`.

- [ ] **Step 4: Полный прогон**

Run: `cargo test --workspace && cargo clippy --workspace -- -D warnings && (cd web && bun test && bun run check)`
Expected: все тесты зелёные, clippy без предупреждений.

- [ ] **Step 5: Проверка лимитов структуры**

Run: `find server/src server/tests web/src web/styles -type f \( -name '*.rs' -o -name '*.ts' -o -name '*.css' -o -name '*.sql' -o -name '*.html' \) | xargs wc -l | awk '$2 != "total" && (($2 ~ /\.css$/ && $1 > 500) || ($2 !~ /\.css$/ && $1 > 250))'`
Expected: пустой вывод.

Run: `find server web/src web/styles deploy -type d -not -path '*/node_modules*' -not -path '*/target*' -not -path '*/dist*' | while read d; do n=$(find "$d" -maxdepth 1 -type f \( -name '*.rs' -o -name '*.ts' -o -name '*.css' -o -name '*.sql' -o -name '*.html' \) | wc -l); [ "$n" -gt 4 ] && echo "$d $n"; done`
Expected: пустой вывод.

- [ ] **Step 6: Commit**

```bash
git add README.md deploy server/tests/e2e.rs
git commit -m "Add deploy files, README and end-to-end test"
```

---

## Вне этого плана

- Перенос существующих страниц из мануала OC2R (`src/main/resources/assets/oc2r/doc/`) в вики. Мануал использует собственный синтаксис ссылок и тегов предметов, поэтому нужен отдельный конвертер, который пишет страницы через `PUT /api/pages/...`. Это следующий план после запуска каркаса.
- Загрузка картинок (таблица `media` создана, маршруты и админка для неё не входят в эти задачи).
- Страница «Мои предложения» для вошедшего пользователя.

## Self-Review (спецификация → задачи)

| Требование | Задача |
|---|---|
| SQLite, миграции, FTS5 | 6, 7 |
| Ревизии, просмотр старых, diff, откат | 2, 3, 7, 10, 12 |
| Удаление и восстановление | 7, 12 |
| Роли, единая политика | 5, 8 |
| API по ключу (read/write/export), хеш, показ один раз | 5, 9, 12 |
| Админка: страницы, предложения, ключи, превью | 12 |
| Предложения, GitHub OAuth, квоты, блокировка | 8, 11, 12 |
| Конфликт при принятии | 2 (`is_conflict`), 12 |
| Рендер на сервере, расширения, очистка HTML | 4 |
| Страницы читателя, плашка старой версии, фолбэк на en | 10 |
| Поиск | 7, 10, 13 |
| Настройки читателя, стиль, пиксель-рамки | 13 |
| Единый `AppError`, коды 401/403/404/422/429 | 1 |
| Деплой, резервная копия | 14 |
| Лимиты 4 файла и 250 строк (CSS 500) | 14 (проверка) |

Известные места, где исполнителю придётся сверяться с документацией версии крейта: API `ammonia::Builder::add_allowed_classes` (Task 4), синтаксис шаблонов `askama` 0.14 (Task 10), сборка CSS и `naming` в `Bun.build` (Task 13). Для них в шагах указаны проверочные тесты и ожидаемое поведение.
