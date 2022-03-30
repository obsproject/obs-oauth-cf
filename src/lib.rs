use worker::*;

mod platforms;
mod utils;

const BLANK_PAGE: &str = "This is an open field west of a white house, with a boarded front door.
There is a small mailbox here.
>";
const OAUTH_FINISHED: &str = "OAuth finished. This window should close momentarily.";

fn handle_redirects(_: Request, ctx: RouteContext<()>) -> Result<Response> {
    let provider = ctx.param("platform");
    let url: String;

    match provider.unwrap().as_str() {
        "twitch" => url = platforms::twitch::get_redirect_url(&ctx),
        "restream" => url = platforms::restream::get_redirect_url(&ctx),
        _ => return Response::error(format!("Unknown provider: {}", provider.unwrap()), 404),
    }

    Response::redirect(Url::parse(&url)?)
}

async fn handle_token(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let provider = ctx.param("platform");
    let form_data = req.form_data().await?;

    match provider.unwrap().as_str() {
        "twitch" => platforms::twitch::get_token(form_data, &ctx).await,
        "restream" => platforms::restream::get_token(form_data, &ctx).await,
        _ => Response::error(format!("Unknown provider: {}", provider.unwrap()), 404),
    }
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    utils::set_panic_hook();
    let router = Router::new();
    router
        .get("/", |_, _| Response::ok(BLANK_PAGE))
        .get("/v1/:platform/redirect", handle_redirects)
        .get("/v1/:platform/finished", |_, _| {
            Response::ok(OAUTH_FINISHED)
        })
        .post_async("/v1/:platform/token", |req, ctx| async move {
            let res = handle_token(req, ctx).await;
            if let Err(res_err) = res {
                Response::error(format!("Bad Request: {}", res_err.to_string()), 400)
            } else {
                res
            }
        })
        .run(req, env)
        .await
}
