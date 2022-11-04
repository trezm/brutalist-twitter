use log::info;
use sqlx::{Pool, Postgres};
use thruster::{
    context::typed_hyper_context::TypedHyperContext, m, middleware::cookies::cookies,
    middleware_fn, App, HyperRequest, MiddlewareNext, MiddlewareResult,
};
use tokio::time::Instant;

use crate::{
    controllers::{
        pages::{home, reply as reply_page, signin, signup, single_tweet},
        tweets::{create_tweet, like_tweet, reply, retweet},
        users::{authenticate, create_user, fetch_user_from_cookie, sign_in_user},
    },
    models::users::User,
};

pub type Ctx = TypedHyperContext<RequestConfig>;

pub struct ServerConfig {
    pub pool: Pool<Postgres>,
}

#[derive(Clone)]
pub struct RequestConfig {
    pub pool: Pool<Postgres>,
    pub user: Option<User>,
}

fn generate_context(request: HyperRequest, state: &ServerConfig, _path: &str) -> Ctx {
    Ctx::new(
        request,
        RequestConfig {
            pool: state.pool.clone(),
            user: None,
        },
    )
}

#[middleware_fn]
async fn ping(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    context.body("pong");

    Ok(context)
}

#[middleware_fn]
async fn profiling(mut context: Ctx, next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    let start_time = Instant::now();

    let method = context
        .hyper_request
        .as_ref()
        .unwrap()
        .request
        .method()
        .clone();
    let path_and_query = context
        .hyper_request
        .as_ref()
        .unwrap()
        .request
        .uri()
        .path_and_query()
        .unwrap()
        .clone();

    context = match next(context).await {
        Ok(context) => context,
        Err(e) => e.context,
    };

    let elapsed_time = start_time.elapsed();
    info!(
        "{}μs\t\t{}\t{}",
        elapsed_time.as_micros(),
        method,
        path_and_query,
    );

    Ok(context)
}

pub async fn app(
    pool: Pool<Postgres>,
) -> Result<App<HyperRequest, Ctx, ServerConfig>, Box<dyn std::error::Error>> {
    Ok(
        App::<HyperRequest, Ctx, ServerConfig>::create(generate_context, ServerConfig { pool })
            .middleware("/", m![profiling])
            .get("/ping", m![ping])
            .get("/", m![cookies, fetch_user_from_cookie, home])
            .get("/signup", m![signup])
            .get("/signin", m![signin])
            .post("/users", m![create_user])
            .post("/sessions", m![sign_in_user])
            .post(
                "/tweets",
                m![cookies, fetch_user_from_cookie, authenticate, create_tweet],
            )
            .get(
                "/tweets/:id",
                m![cookies, fetch_user_from_cookie, single_tweet],
            )
            .post(
                "/tweets/:id/likes",
                m![cookies, fetch_user_from_cookie, authenticate, like_tweet],
            )
            .post(
                "/tweets/:id/retweets",
                m![cookies, fetch_user_from_cookie, authenticate, retweet],
            )
            .get(
                "/tweets/:id/replies",
                m![cookies, fetch_user_from_cookie, authenticate, reply_page],
            )
            .post(
                "/tweets/:id/replies",
                m![cookies, fetch_user_from_cookie, authenticate, reply],
            ),
    )
}
