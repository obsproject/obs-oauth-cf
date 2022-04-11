use worker::*;
mod platforms;
mod utils;

use platforms::restream;
use platforms::twitch;

const BLANK_PAGE: &str = "This is an open field west of a white house, with a boarded front door.
There is a small mailbox here.
>";
const FOUR_OH_FOUR: &str = "This is an open field.
There is nothing here.
>";
const OAUTH_COMPLETE: &str = "OAuth process finished. This window should close momentarily.";

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    utils::set_panic_hook();

    let router = Router::new();
    router
        .get("/", |_, _| Response::ok(BLANK_PAGE))
        .get("/v1/:platform/redirect", handle_redirects)
        .get("/v1/:platform/finalise", |_, _| Response::ok(OAUTH_COMPLETE))
        .post_async("/v1/:platform/token", handle_token)
        // Legacy routes
        .get("/app-auth/:platform", handle_legacy_redirects)
        .post_async("/app-auth/:platform_action", handle_legacy_token)
        .or_else_any_method("/*catchall", |_, _| Response::error(FOUR_OH_FOUR, 404))
        .run(req, env)
        .await
}

fn handle_redirects(_: Request, ctx: RouteContext<()>) -> Result<Response> {
    let platform = ctx.param("platform").unwrap();

    match platform.as_str() {
        "twitch" => twitch::get_redirect(&ctx, false),
        "restream" => restream::get_redirect(&ctx, false),
        _ => Response::error(format!("Unknown platform: {}", platform), 404),
    }
}

async fn handle_token(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let platform = ctx.param("platform").unwrap();

    let form_data = req.form_data().await;
    if let Err(err) = form_data {
        return Response::error(format!("Bad Request: {}", err), 400);
    }

    match platform.as_str() {
        "twitch" => twitch::get_token(&ctx, form_data?, false).await,
        "restream" => restream::get_token(&ctx, form_data?, false).await,
        _ => Response::error(format!("Unknown platform: {}", platform), 404),
    }
}

fn handle_legacy_redirects(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    // Deal with "/app-auth/<platform>[?action=redirect]"
    let platform = ctx.param("platform").unwrap();
    // Redirect if URL param "action" is set to "redirect", otherwise show OAuth complete page
    if req.url()?.query_pairs().any(|(k, v)| k == "action" && v == "redirect") {
        match platform.as_str() {
            "twitch" => twitch::get_redirect(&ctx, true),
            "restream" => restream::get_redirect(&ctx, true),
            _ => Response::error(format!("Unknown platform: {}", platform), 404),
        }
    } else {
        Response::ok(OAUTH_COMPLETE)
    }
}

async fn handle_legacy_token(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    // Deal with "/app-auth/<platform>-token"
    let platform_action = ctx.param("platform_action").unwrap();
    if !platform_action.ends_with("-token") {
        return Response::error("Bad Request", 400);
    }

    let form_data = req.form_data().await;
    if let Err(err) = form_data {
        return Response::error(format!("Bad Request: {}", err), 400);
    }

    let platform = platform_action.split('-').next().unwrap_or_default();
    match platform {
        "twitch" => twitch::get_token(&ctx, form_data?, true).await,
        "restream" => restream::get_token(&ctx, form_data?, true).await,
        _ => Response::error(format!("Unknown platform: {}", platform), 404),
    }
}
