use axum::extract::rejection::FormRejection;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Form, Router};
use reqwest::StatusCode;
use serde::Deserialize;
use tokio::net::TcpListener;

#[derive(Deserialize)]
struct FormData {
    name: String,
    email: String,
}

async fn health_check() -> &'static str {
    "OK"
}

async fn subscribe(form: Result<Form<FormData>, FormRejection>) -> impl IntoResponse {
    let form = match form {
        Ok(form) => form,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, "Invalid form data").into_response();
        }
    };
    StatusCode::OK.into_response()
}

pub async fn run(listener: TcpListener) -> Result<(), std::io::Error> {
    let app = Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe));

    axum::serve(listener, app)
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}
