use newsletter::configuration::{DatabaseSettings, get_configuration};
use newsletter::startup::run;
use sqlx::{Connection, PgConnection, PgPool};
use tokio::net::TcpListener;
use uuid::Uuid;

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
    pub db_name: String,
}

impl TestApp {
    pub async fn cleanup(self) {
        self.db_pool.close().await;

        let mut connection = PgConnection::connect(
            &get_configuration()
                .expect("failed to read configuration")
                .database
                .connection_string_without_db(),
        )
        .await
        .expect("failed to connect to postgres");

        sqlx::query(&format!(r#"DROP DATABASE "{}";"#, self.db_name))
            .execute(&mut connection)
            .await
            .expect("failed to drop test database");
    }
}

async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind to random port");

    let port = listener
        .local_addr()
        .expect("Failed to get local address")
        .port();

    let address = format!("http://127.0.0.1:{}", port);

    let mut configuration = get_configuration().expect("failed to read configuration");
    configuration.database.database_name = Uuid::new_v4().to_string();

    let db_name = configuration.database.database_name.clone();

    let connection_pool = configure_database(&configuration.database).await;

    let connection_pool_clone = connection_pool.clone();
    let _ = tokio::spawn(async move {
        run(listener, connection_pool_clone)
            .await
            .expect("Failed to run server");
    });

    TestApp {
        address,
        db_pool: connection_pool,
        db_name,
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect(&config.connection_string_without_db())
        .await
        .expect("failed to connect to postgres");

    sqlx::query(&format!(
        r#"CREATE DATABASE "{}";"#,
        config.database_name.as_str()
    ))
    .execute(&mut connection)
    .await
    .expect("failed to create database");

    let connection_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("failed to connect to database");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("failed to migrate database");

    connection_pool
}

#[tokio::test]
async fn health_check_works() {
    let app = spawn_app().await;
    let url = format!("{}/health_check", &app.address);
    let client = reqwest::Client::new();

    let response = client
        .get(&url)
        .send()
        .await
        .expect("Failed to send health check request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());

    app.cleanup().await;
}

#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to send subscription request");

    // Assert
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("failed to fetch saved subscription.");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");

    app.cleanup().await;
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to send subscription request");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}",
            error_message
        );
    }

    app.cleanup().await;
}
