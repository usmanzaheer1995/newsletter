use newsletter::configuration::get_configuration;
use newsletter::startup::run;
use sqlx::PgPool;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let configuration = get_configuration().expect("failed to read configuration.");

    let connection = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("failed to connect to database");

    let address = format!("127.0.0.1:{}", configuration.application_port);

    let listener = tokio::net::TcpListener::bind(address).await?;
    run(listener, connection).await
}
