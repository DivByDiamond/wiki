use oc2r_wiki::{config::Config, router};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let cfg = Config::from_env();
    let listener = tokio::net::TcpListener::bind(&cfg.bind)
        .await
        .expect("bind");
    tracing::info!("listening on {}", cfg.bind);
    axum::serve(listener, router()).await.expect("serve");
}
