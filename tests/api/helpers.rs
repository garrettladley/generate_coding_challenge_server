use generate_coding_challenge_server::configuration::{get_configuration, DatabaseSettings};
use generate_coding_challenge_server::startup::run;
use generate_coding_challenge_server::telemetry::{get_subscriber, init_subscriber};
use maplit::hashmap;
use once_cell::sync::Lazy;
use reqwest::Response;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use uuid::Uuid;

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    };
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to random port.");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let mut configuration = get_configuration().expect("Failed to read configuration.");
    configuration.database.database_name = Uuid::new_v4().to_string();

    let connection_pool = configure_database(&configuration.database).await;

    let server =
        run(listener, connection_pool.clone()).expect("Failed to bind address to random port.");
    std::mem::drop(tokio::spawn(server));
    TestApp {
        address,
        db_pool: connection_pool,
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(&*format!(r#"CREATE DATABASE "{}";"#, config.database_name))
        .await
        .expect("Failed to create database.");

    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}

pub async fn register_sample_applicant(client: &reqwest::Client, address: &str) -> Response {
    register_sample_applicant_with_nuid(client, address, "001234567").await
}

pub async fn register_sample_applicant_with_nuid(
    client: &reqwest::Client,
    address: &str,
    nuid: &str,
) -> Response {
    client
        .post(&format!("{}/register", address))
        .json(&hashmap! {
            "name" => "Garrett",
            "nuid" => nuid,
        })
        .send()
        .await
        .expect("Failed to execute request.")
}
