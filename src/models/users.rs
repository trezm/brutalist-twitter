use sqlx::{
    types::chrono::{DateTime, Utc},
    FromRow, Pool, Postgres,
};
use uuid::Uuid;

#[derive(Clone, Debug, FromRow)]
pub struct User {
    pub id: Uuid,
    pub password: String,
    pub username: String,
    pub created_at: DateTime<Utc>,
}

impl User {
    pub async fn create_user(
        pool: &Pool<Postgres>,
        username: &str,
        password_hash: &str,
    ) -> Result<User, sqlx::Error> {
        sqlx::query_as(
            "
            INSERT INTO users (LOWER(username), password)
            VALUES ($1, $2)
            RETURNING id, username, password, created_at",
        )
        .bind(username)
        .bind(&password_hash)
        .fetch_one(pool)
        .await
    }

    pub async fn get_user_for_id(pool: &Pool<Postgres>, id: &Uuid) -> Result<User, sqlx::Error> {
        sqlx::query_as(
            "
            SELECT id, username, password, created_at FROM users WHERE id = $1",
        )
        .bind(id)
        .fetch_one(pool)
        .await
    }

    pub async fn get_user_for_username(
        pool: &Pool<Postgres>,
        username: &str,
    ) -> Result<User, sqlx::Error> {
        sqlx::query_as(
            "
            SELECT id, username, password, created_at FROM users WHERE username = LOWER($1)",
        )
        .bind(username)
        .fetch_one(pool)
        .await
    }
}
