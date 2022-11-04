use app::{Ctx, ServerConfig};
use log::info;
use shuttle_service::error::CustomError;
use sqlx::{Executor, PgPool};
use thruster::{HyperServer, ThrusterServer};

pub mod app;
pub mod controllers;
pub mod models;

#[shuttle_service::main]
async fn shuttle(
    #[shuttle_aws_rds::Postgres] pool: PgPool,
) -> shuttle_service::ShuttleThruster<HyperServer<Ctx, ServerConfig>> {
    info!("Starting server...");

    pool.execute(include_str!("../schema.sql"))
        .await
        .map_err(|e| CustomError::new(e))?;

    let app = app::app(pool)
        .await
        .map_err(|_e| CustomError::msg("Starting thruster server failed"))?;

    info!("Server started...");

    Ok(HyperServer::new(app))
}
