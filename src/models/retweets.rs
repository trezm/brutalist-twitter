use sqlx::{
    types::chrono::{DateTime, Utc},
    FromRow, Pool, Postgres,
};
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct Retweet {
    pub tweet_id: Uuid,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
}

impl Retweet {
    pub async fn create_retweet(
        pool: &Pool<Postgres>,
        tweet_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<Retweet, sqlx::Error> {
        let mut transaction = pool.begin().await?;

        let like = sqlx::query_as(
            "
            INSERT INTO retweets (tweet_id, user_id)
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
        SET retweet_count = retweet_count + 1
        WHERE id = $1",
        )
        .bind(tweet_id)
        .execute(&mut transaction)
        .await?;
        transaction.commit().await?;

        Ok(like)
    }
}
