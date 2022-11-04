use chrono::Duration;
use sqlx::{
    types::chrono::{DateTime, Utc},
    FromRow, Pool, Postgres,
};
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct Tweet {
    pub id: Uuid,
    pub user_id: Uuid,
    pub responding_to: Option<Uuid>,
    pub content: String,
    pub like_count: i64,
    pub retweet_count: i64,
    pub reply_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
pub struct TweetWithUserInfo {
    pub id: Uuid,
    pub user_id: Uuid,
    pub responding_to: Option<Uuid>,
    pub content: String,
    pub username: String,
    pub user_has_retweeted: bool,
    pub user_has_liked: bool,
    pub like_count: i64,
    pub retweet_count: i64,
    pub reply_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Tweet {
    pub async fn create_tweet(
        pool: &Pool<Postgres>,
        user_id: &Uuid,
        responding_to: Option<Uuid>,
        content: String,
    ) -> Result<Tweet, sqlx::Error> {
        let mut transaction = pool.begin().await?;

        let tweet = sqlx::query_as(
            "
            INSERT INTO tweets (user_id, responding_to, content)
            VALUES ($1, $2, $3)
            RETURNING id, user_id, responding_to, content, created_at, updated_at, like_count, retweet_count, reply_count",
        )
        .bind(user_id)
        .bind(responding_to)
        .bind(content)
        .fetch_one(&mut transaction)
        .await?;

        if let Some(tweet_id) = responding_to {
            sqlx::query(
                "
            UPDATE tweets
            SET reply_count = reply_count + 1
            WHERE id = $1",
            )
            .bind(tweet_id)
            .execute(&mut transaction)
            .await?;
        }

        transaction.commit().await?;

        Ok(tweet)
    }

    pub async fn get_tweet_for_id(pool: &Pool<Postgres>, id: &Uuid) -> Result<Tweet, sqlx::Error> {
        sqlx::query_as(
            "
            SELECT id, user_id, responding_to, content, created_at, updated_at, like_count, retweet_count, reply_count FROM tweets WHERE id = $1",
        )
        .bind(id)
        .fetch_one(pool)
        .await
    }

    pub async fn get_tweet_with_user_info(
        pool: &Pool<Postgres>,
        id: &Uuid,
        user_id: Option<&Uuid>,
    ) -> Result<TweetWithUserInfo, sqlx::Error> {
        sqlx::query_as(
            "
            SELECT
                t.id, t.user_id, t.responding_to, t.content,
                t.created_at, t.updated_at, u.username, t.like_count,
                t.retweet_count, t.reply_count,
                l.user_id IS NOT NULL as user_has_liked,
                r.user_id IS NOT NULL as user_has_retweeted
            FROM
                tweets as t
            JOIN
                users as u ON t.user_id = u.id
            LEFT JOIN
                likes as l ON l.tweet_id = t.id AND l.user_id = $2
            LEFT JOIN
                retweets as r ON r.tweet_id = t.id AND r.user_id = $2
            WHERE
                t.id = $1",
        )
        .bind(id)
        .bind(user_id)
        .fetch_one(pool)
        .await
    }

    // pub async fn get_recent_tweets(
    //     pool: &Pool<Postgres>,
    //     offset: Option<DateTime<Utc>>,
    // ) -> Result<Vec<>, sqlx::Error> {
    //     sqlx::query_as(
    //         "
    //         SELECT
    //             t.id, t.user_id, t.responding_to, t.content, t.created_at, t.updated_at, u.username, t.like_count, t.retweet_count, t.reply_count
    //         FROM
    //             tweets as t
    //         JOIN
    //             users as u ON t.user_id = u.id
    //         WHERE
    //                 t.created_at < $1
    //             AND
    //                 t.responding_to IS NULL
    //         ORDER BY
    //             t.created_at DESC
    //         LIMIT 20",
    //     )
    //     .bind(offset.unwrap_or_else(|| Utc::now() + Duration::days(1)))
    //     .fetch_all(pool)
    //     .await
    // }

    pub async fn get_recent_tweets_with_user_info(
        pool: &Pool<Postgres>,
        user_id: Option<&Uuid>,
        offset: Option<DateTime<Utc>>,
    ) -> Result<Vec<TweetWithUserInfo>, sqlx::Error> {
        sqlx::query_as(
            "
            SELECT
                t.id, t.user_id, t.responding_to, t.content,
                t.created_at, t.updated_at, u.username, t.like_count,
                t.retweet_count, t.reply_count,
                l.user_id IS NOT NULL as user_has_liked,
                r.user_id IS NOT NULL as user_has_retweeted
            FROM
                tweets as t
            JOIN
                users as u ON t.user_id = u.id
            LEFT JOIN
                likes as l ON l.tweet_id = t.id AND l.user_id = $2
            LEFT JOIN
                retweets as r ON r.tweet_id = t.id AND r.user_id = $2
            WHERE
                    t.created_at < $1
                AND
                    t.responding_to IS NULL
        ORDER BY
                t.created_at DESC
            LIMIT 20",
        )
        .bind(offset.unwrap_or_else(|| Utc::now() + Duration::days(1)))
        .bind(user_id)
        .fetch_all(pool)
        .await
    }

    pub async fn get_tweet_replies_with_user_info(
        pool: &Pool<Postgres>,
        tweet_id: &Uuid,
        user_id: Option<&Uuid>,
        offset: Option<DateTime<Utc>>,
    ) -> Result<Vec<TweetWithUserInfo>, sqlx::Error> {
        sqlx::query_as(
            "
            SELECT
                t.id, t.user_id, t.responding_to, t.content,
                t.created_at, t.updated_at, u.username, t.like_count,
                t.retweet_count, t.reply_count,
                l.user_id IS NOT NULL as user_has_liked,
                r.user_id IS NOT NULL as user_has_retweeted
            FROM
                tweets as t
            JOIN
                users as u ON t.user_id = u.id
            LEFT JOIN
                likes as l ON l.tweet_id = t.id AND l.user_id = $3
            LEFT JOIN
                retweets as r ON r.tweet_id = t.id AND r.user_id = $3
            WHERE
                    t.created_at < $2
                AND
                    t.responding_to = $1
            ORDER BY
                t.created_at DESC
            LIMIT 20",
        )
        .bind(tweet_id)
        .bind(offset.unwrap_or_else(|| Utc::now() + Duration::days(1)))
        .bind(user_id)
        .fetch_all(pool)
        .await
    }
}
