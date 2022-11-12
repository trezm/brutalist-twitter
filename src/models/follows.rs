use sqlx::{
    types::chrono::{DateTime, Utc},
    FromRow, Pool, Postgres,
};
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct Follow {
    pub follower_id: Uuid,
    pub following_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
pub struct Follows {
    pub count: i64,
}

impl Follow {
    pub async fn create_follow(
        pool: &Pool<Postgres>,
        follower_id: &Uuid,
        following_id: &Uuid,
    ) -> Result<Follow, sqlx::Error> {
        sqlx::query_as(
            "
            INSERT INTO follows (follower_id, following_id)
            VALUES ($1, $2)
            RETURNING follower_id, following_id, created_at",
        )
        .bind(follower_id)
        .bind(following_id)
        .fetch_one(pool)
        .await
    }

    pub async fn get_follow_count(
        pool: &Pool<Postgres>,
        following_id: &Uuid,
    ) -> Result<i64, sqlx::Error> {
        let follows: Follows = sqlx::query_as(
            "
            SELECT COUNT(follower_id) as follows FROM follows WHERE following_id = $1",
        )
        .bind(following_id)
        .fetch_one(pool)
        .await?;

        Ok(follows.count)
    }
}
