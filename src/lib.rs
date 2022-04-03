use worker::*;
mod platforms;
mod utils;

use platforms::restream;
use platforms::twitch;

const BLANK_PAGE: &str = "This is an open field west of a white house, with a boarded front door.
There is a small mailbox here.
>";
const OAUTH_FINISHED: &str = "OAuth finished. This window should close momentarily.";

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    utils::set_panic_hook();
    let router = Router::new();
    router
        .get("/", |_, _| Response::ok(BLANK_PAGE))
        .get("/v1/:platform/redirect", handle_redirects)
        .get("/v1/:platform/finalise", |_, _| {
            Response::ok(OAUTH_FINISHED)
        })
        .post_async("/v1/:platform/token", handle_token)
        // Legacy routes
        .get("/app-auth/:platform", handle_legacy_redirects)
        .post_async("/app-auth/:platform_action", handle_legacy_token)
        .run(req, env)
        .await
}

fn handle_redirects(_: Request, ctx: RouteContext<()>) -> Result<Response> {
    let platform = ctx.param("platform");
    let url: String;

    match platform.unwrap().as_str() {
        "twitch" => url = twitch::get_redirect_url(&ctx, false),
        "restream" => url = restream::get_redirect_url(&ctx, false),
        _ => return Response::error(format!("Unknown platform: {}", platform.unwrap()), 404),
    }

    Response::redirect(Url::parse(&url)?)
}

async fn handle_token(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let platform = ctx.param("platform");
    let form_data = req.form_data().await?;

    match platform.unwrap().as_str() {
        "twitch" => twitch::get_token(&ctx, form_data, false).await,
        "restream" => restream::get_token(&ctx, form_data, false).await,
        _ => Response::error(format!("Unknown platform: {}", platform.unwrap()), 404),
    }
}

fn handle_legacy_redirects(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    // Deal with "/app-auth/<platform>?action=redirect"
    let platform = ctx.param("platform").unwrap();
    // Redirect if URL param "action" is set
    let mut do_redirect = false;
    // There really must be a better way to do this...
    req.url().unwrap().query_pairs().for_each(|(k, v)| {
        if k == "action" && v == "redirect" {
            do_redirect = true;
        }
    });

    if do_redirect {
        let url: String;

        match platform.as_str() {
            "twitch" => url = twitch::get_redirect_url(&ctx, true),
            "restream" => url = restream::get_redirect_url(&ctx, true),
            _ => return Response::error(format!("Unknown platform: {}", platform), 404),
        }

        Response::redirect(Url::parse(&url)?)
    } else {
        Response::ok(OAUTH_FINISHED)
    }
}

async fn handle_legacy_token(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    // Deal with "/app-auth/<platform>-token"
    let platform_action = ctx.param("platform_action").unwrap();
    if !platform_action.contains("-") {
        return Response::error("Bad Request", 400);
    }

    let parts = platform_action.split("-").collect::<Vec<&str>>();
    let platform = parts[0];
    let suffix = parts[1];

    if suffix == "token" {
        let form_data = req.form_data().await?;
        match platform {
            "twitch" => twitch::get_token(&ctx, form_data, true).await,
            "restream" => restream::get_token(&ctx, form_data, true).await,
            _ => Response::error(format!("Unknown platform: {}", platform), 404),
        }
    } else {
        Response::error(format!("Unknown suffix: {}", suffix), 400)
    }
}
