use oc2r_wiki::{config::Config, http::state::AppState, router, store::db};
use std::io::IsTerminal;
use std::net::SocketAddr;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_ansi(std::io::stdout().is_terminal())
        .init();
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!(
            "oc2r-wiki [--sync-content]\n\nWithout arguments: initialize empty database and serve.\n--sync-content: import bundled content into WIKI_DB and exit; preserve editor changes."
        );
        return;
    }
    let sync = args.as_slice() == ["--sync-content"];
    if !args.is_empty() && !sync {
        eprintln!("unknown arguments; use --help");
        std::process::exit(1);
    }
    let cfg = match Config::from_env() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("invalid configuration: {e}");
            std::process::exit(1);
        }
    };
    let db = match db::open(&cfg.db_path) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("cannot open database {}: {}", cfg.db_path, e.0);
            std::process::exit(1);
        }
    };
    let result = oc2r_wiki::store::catalog::bundled()
        .and_then(|catalog| oc2r_wiki::store::catalog::import(&db, &catalog, !sync));
    match result {
        Ok(report) => tracing::info!(?report, "content bundle"),
        Err(e) => {
            eprintln!("cannot import content: {}", e.0);
            std::process::exit(1);
        }
    }
    if sync {
        return;
    }
    let listener = match tokio::net::TcpListener::bind(&cfg.bind).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("cannot bind {}: {e}", cfg.bind);
            std::process::exit(1);
        }
    };
    tracing::info!("listening on {}", cfg.bind);
    if let Err(e) = axum::serve(
        listener,
        router(AppState::new(db).with_config(cfg.clone()))
            .into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    {
        eprintln!("server error: {e}");
        std::process::exit(1);
    }
}
