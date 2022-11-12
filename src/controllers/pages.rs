use std::str::FromStr;

use askama::Template;
use log::{error, info};
use thruster::{
    context::context_ext::ContextExt,
    errors::{ErrorSet, ThrusterError},
    middleware_fn, Context, MiddlewareNext, MiddlewareResult,
};
use uuid::Uuid;

use crate::{
    app::Ctx,
    models::{
        tweets::{Tweet, TweetWithUserInfo},
        users::User,
    },
};

#[derive(Template)]
#[template(path = "signup.html")]
pub struct SignUp;

#[middleware_fn]
pub async fn signup(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    context.set("Content-Type", "text/html");
    context.body(&SignUp.render().unwrap());

    Ok(context)
}

#[derive(Template)]
#[template(path = "signin.html")]
pub struct SignIn;

#[middleware_fn]
pub async fn signin(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    context.set("Content-Type", "text/html");
    context.body(&SignIn.render().unwrap());

    Ok(context)
}

#[derive(Template)]
#[template(path = "home.html")]
pub struct Feed<'a> {
    user: Option<&'a User>,
    feed: Vec<TweetWithUserInfo>,
}

#[middleware_fn]
pub async fn home(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    let user = context.extra.user.clone();
    let user_id = context.extra.user.clone().map(|v| v.id);

    context.set("Content-Type", "text/html");
    context.body(
        &Feed {
            user: user.as_ref(),
            feed: Tweet::get_recent_tweets_with_user_info(
                &context.extra.pool,
                user_id.as_ref(),
                None,
            )
            .await
            .map_err(|_e| {
                error!("_e: {:#?}", _e);

                ThrusterError::generic_error(Ctx::new_without_request(context.extra.clone()))
            })?,
        }
        .render()
        .unwrap(),
    );

    Ok(context)
}

#[derive(Template)]
#[template(path = "reply.html")]
pub struct ReplyTo {
    user: User,
    tweet: TweetWithUserInfo,
}

#[middleware_fn]
pub async fn reply(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    let tweet_id = context
        .params()
        .get("id")
        .and_then(|id_string| Uuid::from_str(&id_string.param).ok())
        .ok_or(ThrusterError::generic_error(Ctx::new_without_request(
            context.extra.clone(),
        )))?;
    let user = context.extra.user.take().unwrap();
    let tweet = Tweet::get_tweet_with_user_info(&context.extra.pool, &tweet_id, Some(&user.id))
        .await
        .map_err(|_e| {
            error!("_e: {:#?}", _e);
            ThrusterError::generic_error(Ctx::new_without_request(context.extra.clone()))
        })?;

    context.set("Content-Type", "text/html");
    context.body(&ReplyTo { user, tweet }.render().unwrap());

    Ok(context)
}

#[derive(Template)]
#[template(path = "single_tweet.html")]
pub struct SingleTweet<'a> {
    user: Option<&'a User>,
    tweet: TweetWithUserInfo,
    replies: Vec<TweetWithUserInfo>,
}

#[middleware_fn]
pub async fn single_tweet(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    let user_id = context.extra.user.as_ref().map(|v| v.id.clone());

    let tweet_id = context
        .params()
        .get("id")
        .and_then(|id_string| Uuid::from_str(&id_string.param).ok())
        .ok_or(ThrusterError::generic_error(Ctx::new_without_request(
            context.extra.clone(),
        )))?;
    let tweet = Tweet::get_tweet_with_user_info(&context.extra.pool, &tweet_id, user_id.as_ref())
        .await
        .map_err(|_e| {
            error!("_e: {:#?}", _e);
            ThrusterError::generic_error(Ctx::new_without_request(context.extra.clone()))
        })?;
    let replies = Tweet::get_tweet_replies_with_user_info(
        &context.extra.pool,
        &tweet_id,
        user_id.as_ref(),
        None,
    )
    .await
    .map_err(|_e| {
        error!("_e: {:#?}", _e);
        ThrusterError::generic_error(Ctx::new_without_request(context.extra.clone()))
    })?;

    context.set("Content-Type", "text/html");
    context.body(
        &SingleTweet {
            user: context.extra.user.as_ref(),
            tweet,
            replies,
        }
        .render()
        .unwrap(),
    );

    Ok(context)
}

#[derive(Template)]
#[template(path = "user.html")]
pub struct UserPage<'a> {
    user: Option<&'a User>,
    page_user: User,
    feed: Vec<TweetWithUserInfo>,
    following_count: u64,
    foller_count: u64,
}

#[middleware_fn]
pub async fn user_page(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    let user = context.extra.user.clone();
    let user_id = context.extra.user.clone().map(|v| v.id);

    let page_user_id = context
        .params()
        .get("id")
        .and_then(|id_string| Uuid::from_str(&id_string.param).ok())
        .ok_or(ThrusterError::generic_error(Ctx::new_without_request(
            context.extra.clone(),
        )))?;
    let page_user = User::get_user_for_id(&context.extra.pool, &page_user_id)
        .await
        .map_err(|_e| {
            error!("_e: {:#?}", _e);
            ThrusterError::generic_error(Ctx::new_without_request(context.extra.clone()))
        })?;

    context.set("Content-Type", "text/html");
    context.body(
        &UserPage {
            user: user.as_ref(),
            feed: Tweet::get_recent_tweets_with_user_info(
                &context.extra.pool,
                user_id.as_ref(),
                None,
            )
            .await
            .map_err(|_e| {
                error!("_e: {:#?}", _e);
                ThrusterError::generic_error(Ctx::new_without_request(context.extra.clone()))
            })?,
            page_user,
        }
        .render()
        .unwrap(),
    );

    Ok(context)
}
