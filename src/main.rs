use newsletter::configuration::get_configuration;
use newsletter::startup::run;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let configuration = get_configuration().expect("failed to read configuration.");

    let address = format!("127.0.0.1:{}", configuration.application_port);

    let listener = tokio::net::TcpListener::bind(address).await?;
    run(listener).await
}
