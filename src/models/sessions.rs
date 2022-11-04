use sha2::{Digest, Sha256};
use sqlx::{
    types::chrono::{DateTime, Utc},
    FromRow, Pool, Postgres,
};
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct Session {
    pub id: Uuid,
    pub token: String,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
}

impl Session {
    pub async fn create_session(
        pool: &Pool<Postgres>,
        user_id: &Uuid,
    ) -> Result<Session, sqlx::Error> {
        sqlx::query_as(
            "
            INSERT INTO sessions (token, user_id)
            VALUES ($1, $2)
            RETURNING id, token, user_id, created_at",
        )
        .bind(format!(
            "{:x}",
            Sha256::new()
                .chain_update(Uuid::new_v4().to_bytes_le())
                .finalize()
        ))
        .bind(user_id)
        .fetch_one(pool)
        .await
    }

    pub async fn get_session_from_token(
        pool: &Pool<Postgres>,
        token: &str,
    ) -> Result<Session, sqlx::Error> {
        sqlx::query_as(
            "
            SELECT id, token, user_id, created_at FROM sessions WHERE token = $1",
        )
        .bind(token)
        .fetch_one(pool)
        .await
    }
}
