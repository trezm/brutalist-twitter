use sqlx::{
    types::chrono::{DateTime, Utc},
    FromRow, Pool, Postgres,
};
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct Like {
    pub tweet_id: Uuid,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
}

impl Like {
    pub async fn create_like(
        pool: &Pool<Postgres>,
        tweet_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<Like, sqlx::Error> {
        let mut transaction = pool.begin().await?;

        let like = sqlx::query_as(
            "
            INSERT INTO likes (tweet_id, user_id)
            VALUES ($1, $2)
            RETURNING tweet_id, user_id, created_at",
        )
        .bind(tweet_id)
        .bind(user_id)
        .fetch_one(&mut transaction)
        .await?;

        sqlx::query(
            "
        UPDATE tweets
        SET like_count = like_count + 1
        WHERE id = $1",
        )
        .bind(tweet_id)
        .execute(&mut transaction)
        .await?;
        transaction.commit().await?;

        Ok(like)
    }

    pub async fn delete_like(
        pool: &Pool<Postgres>,
        tweet_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<(), sqlx::Error> {
        let mut transaction = pool.begin().await?;

        sqlx::query(
            "
            DELETE FROM likes WHERE tweet_id = $1 AND user_id = $2",
        )
        .bind(tweet_id)
        .bind(user_id)
        .execute(&mut transaction)
        .await?;

        sqlx::query(
            "
        UPDATE tweets
        SET like_count = like_count - 1
        WHERE id = $1",
        )
        .bind(tweet_id)
        .execute(&mut transaction)
        .await?;

        Ok(())
    }
}
