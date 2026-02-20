use axum::Form;
use axum::extract::rejection::FormRejection;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

pub async fn subscribe(form: Result<Form<FormData>, FormRejection>) -> impl IntoResponse {
    let form = match form {
        Ok(form) => form,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, "Invalid form data").into_response();
        }
    };
    StatusCode::OK.into_response()
}
