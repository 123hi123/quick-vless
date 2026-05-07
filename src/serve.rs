use anyhow::Result;
use axum::{
    extract::Path,
    http::{StatusCode, header},
    response::IntoResponse,
    routing::get,
    Router,
};
use std::net::SocketAddr;

use crate::config::AppConfig;

pub async fn run_server(config: &AppConfig) -> Result<()> {
    let app = Router::new().route("/sub/{token}", get(serve_subscription));

    let addr = SocketAddr::from(([0, 0, 0, 0], config.sub_port));
    println!("Subscription server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn serve_subscription(Path(token): Path<String>) -> impl IntoResponse {
    let path = format!("{}/subs/{}.yaml", AppConfig::config_dir(), token);

    match tokio::fs::read_to_string(&path).await {
        Ok(content) => (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, "text/yaml; charset=utf-8"),
                (
                    header::CONTENT_DISPOSITION,
                    "attachment; filename=\"clash.yaml\"",
                ),
            ],
            content,
        )
            .into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}
