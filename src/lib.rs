use worker::*;

mod utils;
mod platforms;

fn log_request(req: &Request) {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or("unknown region".into())
    );
}

const BLANK_PAGE: &str = "This is an open field west of a white house, with a boarded front door.
There is a small mailbox here.
>";
const OAUTH_FINISHED : &str = "OAuth finished. This window should close momentarily.";

fn handle_redirects(_: Request, ctx: RouteContext<()>) -> Result<Response> {
    let provider = ctx.param("platform");
    let mut url: Option<String> = None;

    match provider.unwrap().as_str() {
        "twitch" => url = Some(platforms::twitch::get_redirect_url(&ctx)),
        _ => {}
    }

    if url.is_some() {
        Response::redirect(Url::parse(&url.unwrap()).unwrap())
    } else {
        Response::error(format!("Unknown provider: {}", provider.unwrap()), 404)
    }
}

async fn handle_token(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let provider = ctx.param("platform");
    let params = req.form_data().await?;

    match provider.unwrap().as_str() {
        "twitch" => platforms::twitch::get_token(params, &ctx).await,
        _ => Response::error(format!("Unknown provider: {}", provider.unwrap()), 404)
    }
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    log_request(&req);
    utils::set_panic_hook();
    let router = Router::new();
    router
        .get("/", |_, _| Response::ok(BLANK_PAGE))
        .get("/v1/:platform/redirect", handle_redirects)
        .get("/v1/:platform/finished", |_, _| Response::ok(OAUTH_FINISHED))
        .post_async("/v1/:platform/token", |req, ctx| async move {
            let res = handle_token(req, ctx).await;
            if let Err(_res) = res {
                Response::error("Bad Request (Fuck)", 400)
            } else {
                res
            }
        })
        .run(req, env)
        .await
}
