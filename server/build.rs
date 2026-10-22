//! `rust-embed` (see `src/http/assets.rs`) fails to compile when the embedded
//! folder `../web/dist` does not exist. That folder is gitignored build output
//! of `cd web && bun run build`, so on a clean checkout it is missing. This
//! script runs nothing: it only creates the empty directory so `cargo test`
//! works without bun. Without a frontend build `/static/*` simply returns 404.
fn main() {
    let dist = std::path::Path::new("../web/dist");
    if let Err(e) = std::fs::create_dir_all(dist) {
        println!("cargo:warning=cannot create {}: {e}", dist.display());
    }
    println!("cargo:rerun-if-changed=../web/dist");
}
