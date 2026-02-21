use axum::Form;
use axum::extract::State;
use axum::extract::rejection::FormRejection;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

pub async fn subscribe(
    State(pool): State<PgPool>,
    form: Result<Form<FormData>, FormRejection>,
) -> impl IntoResponse {
    let form = match form {
        Ok(form) => form,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, "Invalid form data").into_response();
        }
    };

    match sqlx::query!(
        "
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
    ",
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(&pool)
    .await
    {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => {
            println!("failed to execute query: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Invalid form data").into_response()
        }
    }
}
