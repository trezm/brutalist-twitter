use std::str::FromStr;

use log::error;
use serde::Deserialize;
use thruster::{
    context::context_ext::ContextExt,
    errors::{ErrorSet, ThrusterError},
    middleware::cookies::HasCookies,
    middleware_fn, MiddlewareNext, MiddlewareResult,
};
use uuid::Uuid;

use crate::{
    app::Ctx,
    models::{likes::Like, retweets::Retweet, tweets::Tweet},
};

#[derive(Deserialize)]
pub struct CreateTweetReq {
    pub content: String,
}

#[middleware_fn]
pub async fn create_tweet(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    let CreateTweetReq { content } =
        serde_urlencoded::from_str(&context.body_string().await.map_err(|_e| {
            ThrusterError::parsing_error(
                Ctx::new_without_request(context.extra.clone()),
                "Bad request",
            )
        })?)
        .map_err(|_e| {
            ThrusterError::parsing_error(
                Ctx::new_without_request(context.extra.clone()),
                "Bad request",
            )
        })?;

    Tweet::create_tweet(
        &context.extra.pool,
        &context.extra.user.as_ref().unwrap().id,
        None,
        content,
    )
    .await
    .map_err(|_e| ThrusterError::generic_error(Ctx::new_without_request(context.extra.clone())))?;

    context.redirect("/");

    Ok(context)
}

#[derive(Deserialize)]
pub struct ReplyReq {
    pub content: String,
}

#[middleware_fn]
pub async fn reply(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    let responding_to = context
        .params()
        .get("id")
        .and_then(|id_string| Uuid::from_str(&id_string.param).ok())
        .ok_or(ThrusterError::generic_error(Ctx::new_without_request(
            context.extra.clone(),
        )))?;

    let ReplyReq { content } =
        serde_urlencoded::from_str(&context.body_string().await.map_err(|_e| {
            error!("_e: {:#?}", _e);

            ThrusterError::parsing_error(
                Ctx::new_without_request(context.extra.clone()),
                "Bad request",
            )
        })?)
        .map_err(|_e| {
            error!("_e: {:#?}", _e);

            ThrusterError::parsing_error(
                Ctx::new_without_request(context.extra.clone()),
                "Bad request",
            )
        })?;

    Tweet::create_tweet(
        &context.extra.pool,
        &context.extra.user.as_ref().unwrap().id,
        Some(responding_to),
        content,
    )
    .await
    .map_err(|_e| {
        error!("_e: {:#?}", _e);
        ThrusterError::generic_error(Ctx::new_without_request(context.extra.clone()))
    })?;

    context.redirect("/");

    Ok(context)
}

#[middleware_fn]
pub async fn like_tweet(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    let tweet_id = context
        .params()
        .get("id")
        .and_then(|id_string| Uuid::from_str(&id_string.param).ok())
        .ok_or(ThrusterError::generic_error(Ctx::new_without_request(
            context.extra.clone(),
        )))?;

    Like::create_like(
        &context.extra.pool,
        &tweet_id,
        &context.extra.user.as_ref().unwrap().id,
    )
    .await
    .map_err(|_e| {
        error!("_e: {:#?}", _e);
        ThrusterError::generic_error(Ctx::new_without_request(context.extra.clone()))
    })?;

    let location = context
        .get_header("Referer")
        .pop()
        .unwrap_or_else(|| "/".to_string())
        .to_owned();

    context.redirect(&location);

    Ok(context)
}

#[middleware_fn]
pub async fn retweet(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    let tweet_id = context
        .params()
        .get("id")
        .and_then(|id_string| Uuid::from_str(&id_string.param).ok())
        .ok_or(ThrusterError::generic_error(Ctx::new_without_request(
            context.extra.clone(),
        )))?;

    Retweet::create_retweet(
        &context.extra.pool,
        &tweet_id,
        &context.extra.user.as_ref().unwrap().id,
    )
    .await
    .map_err(|_e| {
        error!("_e: {:#?}", _e);
        ThrusterError::generic_error(Ctx::new_without_request(context.extra.clone()))
    })?;

    let location = context
        .get_header("Referer")
        .pop()
        .unwrap_or_else(|| "/".to_string())
        .to_owned();

    context.redirect(&location);

    Ok(context)
}
