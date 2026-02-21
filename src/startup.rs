use crate::routes::{health_check, subscribe};
use axum::Router;
use axum::routing::{get, post};
use sqlx::PgPool;
use tokio::net::TcpListener;

pub async fn run(listener: TcpListener, pool: PgPool) -> Result<(), std::io::Error> {
    let app = Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .with_state(pool);

    axum::serve(listener, app)
        .await
        .map_err(std::io::Error::other)
}
