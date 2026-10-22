//! `rust-embed` (see `src/http/assets.rs`) fails to compile when the embedded
//! folder `../web/dist` does not exist. That folder is gitignored output of
//! `cd web && bun run build`.
//!
//! Release builds ship the site, so they fail loudly when `app.js` or
//! `app.css` is missing. Debug builds only warn and create an empty folder so
//! `cargo check` works without bun (`/static/*` then answers 404).
use std::path::Path;

const REQUIRED: [&str; 2] = ["app.js", "app.css"];

fn main() {
    let dist = Path::new("../web/dist");
    let missing: Vec<&str> = REQUIRED
        .into_iter()
        .filter(|f| !dist.join(f).is_file())
        .collect();
    if !missing.is_empty() {
        let msg = format!(
            "web/dist is missing {}: run `bun run build` in web/ before building the server",
            missing.join(", ")
        );
        if std::env::var("PROFILE").as_deref() == Ok("release") {
            panic!("{msg}");
        }
        println!("cargo:warning={msg}");
        if let Err(e) = std::fs::create_dir_all(dist) {
            println!("cargo:warning=cannot create {}: {e}", dist.display());
        }
    }
    println!("cargo:rerun-if-changed=../web/dist");
}
