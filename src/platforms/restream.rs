use worker::{FormData, Response, Result, RouteContext};

use crate::platforms::oauth;
use crate::platforms::oauth::OAuthConfig;

const SCOPES: &str = "stream.default.read chat.default.read";
const RESTREAM_AUTH_URL: &str = "https://api.restream.io/login";
const RESTREAM_TOKEN_URL: &str = "https://api.restream.io/oauth/token";

pub fn get_redirect(ctx: &RouteContext<()>, legacy: bool) -> Result<Response> {
    match get_restream_config(ctx, legacy) {
        Ok(config) => oauth::get_redirect(config),
        Err(err) => Response::error(format!("Something went wrong: {}", err), 500),
    }
}

pub async fn get_token(ctx: &RouteContext<()>, form_data: FormData, legacy: bool) -> Result<Response> {
    match get_restream_config(ctx, legacy) {
        Ok(config) => oauth::get_token(config, form_data).await,
        Err(err) => Response::error(format!("Something went wrong: {}", err), 500),
    }
}

fn get_restream_config(ctx: &RouteContext<()>, legacy: bool) -> Result<OAuthConfig> {
    let mut config = oauth::OAuthConfig {
        name: "Restream".to_string(),
        client_id: ctx.secret("RESTREAM_ID")?.to_string(),
        client_secret: ctx.secret("RESTREAM_SECRET")?.to_string(),
        redirect_url: ctx.var("RESTREAM_REDIRECT_URL")?.to_string(),
        scope: SCOPES.to_string(),
        auth_url: RESTREAM_AUTH_URL.to_string(),
        token_url: RESTREAM_TOKEN_URL.to_string(),
        ..Default::default()
    };

    if legacy {
        config.redirect_url = ctx.var("RESTREAM_LEGACY_REDIRECT_URL")?.to_string();
    }

    config
        .extra_params
        .insert("include_granted_scopes".to_string(), "true".to_string());

    Ok(config)
}
