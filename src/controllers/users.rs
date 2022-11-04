use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use log::error;
use serde::Deserialize;
use thruster::{
    errors::{ErrorSet, ThrusterError},
    middleware::cookies::CookieOptions,
    middleware_fn, MiddlewareNext, MiddlewareResult,
};

use crate::{
    app::Ctx,
    models::{sessions::Session, users::User},
};

#[derive(Deserialize)]
pub struct CreatUserReq {
    pub username: String,
    pub password: String,
}

#[middleware_fn]
pub async fn create_user(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    let CreatUserReq { username, password } =
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

    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(password.as_bytes(), salt.as_ref())
        .map(|h| h.to_string())
        .map_err(|_e| {
            error!("_e: {:#?}", _e);

            ThrusterError::parsing_error(
                Ctx::new_without_request(context.extra.clone()),
                "Bad request",
            )
        })?
        .to_string();

    let user = User::create_user(&context.extra.pool, &username, &password_hash)
        .await
        .map_err(|_e| {
            error!("_e: {:#?}", _e);

            ThrusterError::generic_error(Ctx::new_without_request(context.extra.clone()))
        })?;

    let session = Session::create_session(&context.extra.pool, &user.id)
        .await
        .map_err(|_e| {
            error!("_e: {:#?}", _e);

            ThrusterError::generic_error(Ctx::new_without_request(context.extra.clone()))
        })?;

    context.redirect("/");
    context.cookie(
        "Session",
        &session.token,
        &CookieOptions {
            http_only: true,
            ..CookieOptions::default()
        },
    );

    Ok(context)
}

#[derive(Deserialize)]
pub struct SignInReq {
    pub username: String,
    pub password: String,
}

#[middleware_fn]
pub async fn sign_in_user(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    let SignInReq { username, password } =
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

    let user = User::get_user_for_username(&context.extra.pool, &username)
        .await
        .map_err(|_e| {
            error!("_e: {:#?}", _e);

            ThrusterError::generic_error(Ctx::new_without_request(context.extra.clone()))
        })?;

    if Argon2::default()
        .verify_password(
            password.as_bytes(),
            &PasswordHash::new(&user.password).unwrap(),
        )
        .is_ok()
    {
        let session = Session::create_session(&context.extra.pool, &user.id)
            .await
            .map_err(|_e| {
                error!("_e: {:#?}", _e);

                ThrusterError::generic_error(Ctx::new_without_request(context.extra.clone()))
            })?;

        context.redirect("/");
        context.cookie(
            "Session",
            &session.token,
            &CookieOptions {
                http_only: true,
                ..CookieOptions::default()
            },
        );
    } else {
        return Err(ThrusterError::unauthorized_error(context));
    }

    Ok(context)
}

#[middleware_fn]
pub async fn fetch_user_from_cookie(
    mut context: Ctx,
    next: MiddlewareNext<Ctx>,
) -> MiddlewareResult<Ctx> {
    let session_token = context.cookies.get("Session");

    if let Some(session_token) = session_token {
        let session =
            Session::get_session_from_token(&context.extra.pool, &session_token.value).await;

        if let Ok(session) = session {
            context.extra.user = User::get_user_for_id(&context.extra.pool, &session.user_id)
                .await
                .ok();
        } else {
            context.cookie(
                "Session",
                "",
                &CookieOptions {
                    http_only: true,
                    expires: 1,
                    ..CookieOptions::default()
                },
            );
        }
    }

    next(context).await
}

#[middleware_fn]
pub async fn authenticate(context: Ctx, next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    if let None = context.extra.user.as_ref() {
        Err(ThrusterError::unauthorized_error(context))
    } else {
        next(context).await
    }
}
