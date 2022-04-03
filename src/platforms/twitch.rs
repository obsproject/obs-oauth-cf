use worker::{FormData, Response, Result, RouteContext};

use crate::platforms::oauth;
use crate::platforms::oauth::OAuthConfig;

const SCOPES: &str = "channel:read:stream_key";
const TWITCH_AUTH_URL: &str = "https://id.twitch.tv/oauth2/authorize";
const TWITCH_TOKEN_URL: &str = "https://id.twitch.tv/oauth2/token";

pub fn get_redirect(ctx: &RouteContext<()>, legacy: bool) -> Result<Response> {
    match get_twitch_config(ctx, legacy) {
        Ok(config) => oauth::get_redirect(config),
        Err(err) => Response::error(format!("Something went wrong: {}", err), 500),
    }
}

pub async fn get_token(
    ctx: &RouteContext<()>,
    form_data: FormData,
    legacy: bool,
) -> Result<Response> {
    match get_twitch_config(ctx, legacy) {
        Ok(config) => oauth::get_token(config, form_data).await,
        Err(err) => Response::error(format!("Something went wrong: {}", err), 500),
    }
}

pub fn get_twitch_config(ctx: &RouteContext<()>, legacy: bool) -> Result<OAuthConfig> {
    let mut config = oauth::OAuthConfig {
        name: "Twitch".to_string(),
        client_id: ctx.secret("TWITCH_ID")?.to_string(),
        client_secret: ctx.secret("TWITCH_SECRET")?.to_string(),
        redirect_url: ctx.var("TWITCH_REDIRECT_URL")?.to_string(),
        scope: SCOPES.to_string(),
        auth_url: TWITCH_AUTH_URL.to_string(),
        token_url: TWITCH_TOKEN_URL.to_string(),
        ..Default::default()
    };

    if legacy {
        config.redirect_url = ctx.var("TWITCH_LEGACY_REDIRECT_URL")?.to_string();
    }

    Ok(config)
}
