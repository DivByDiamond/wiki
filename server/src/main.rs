use oc2r_wiki::{config::Config, http::state::AppState, router, store::db};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let cfg = Config::from_env();
    let db = match db::open(&cfg.db_path) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("cannot open database {}: {}", cfg.db_path, e.0);
            std::process::exit(1);
        }
    };
    let listener = match tokio::net::TcpListener::bind(&cfg.bind).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("cannot bind {}: {e}", cfg.bind);
            std::process::exit(1);
        }
    };
    tracing::info!("listening on {}", cfg.bind);
    if let Err(e) = axum::serve(listener, router(AppState::new(db))).await {
        eprintln!("server error: {e}");
        std::process::exit(1);
    }
}
