use log::info;
use sqlx::{postgres::PgPoolOptions, Executor};
use thruster::{hyper_server::HyperServer, ThrusterServer};

pub mod app;
pub mod controllers;
pub mod models;

#[tokio::main]
async fn main() {
    env_logger::init();
    info!("Starting server...");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres:password@localhost/brutalist-twitter")
        .await
        .expect("Could not create postgres connection pool");

    pool.execute(include_str!("../schema.sql"))
        .await
        .expect("Could not create schema in database");

    let app = app::app(pool).await.expect("Could not create app");

    info!("Server started...");

    let server = HyperServer::new(app);
    server.build("0.0.0.0", 4321).await;
}
