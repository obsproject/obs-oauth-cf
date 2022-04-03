use worker::{FormData, Response, Result, RouteContext};

use crate::platforms::oauth;
use crate::platforms::oauth::OAuthConfig;

const SCOPES: &str = "channel:read:stream_key";
const TWITCH_AUTH_URL: &str = "https://id.twitch.tv/oauth2/authorize";
const TWITCH_TOKEN_URL: &str = "https://id.twitch.tv/oauth2/token";

pub fn get_redirect_url(ctx: &RouteContext<()>, legacy: bool) -> String {
    oauth::get_redirect_url(get_config(ctx, legacy))
}

pub async fn get_token(
    ctx: &RouteContext<()>,
    form_data: FormData,
    legacy: bool,
) -> Result<Response> {
    oauth::get_token(get_config(ctx, legacy), form_data).await
}

fn get_config(ctx: &RouteContext<()>, legacy: bool) -> OAuthConfig {
    let mut config = oauth::OAuthConfig {
        name: "Twitch".to_string(),
        client_id: ctx.secret("TWITCH_ID").unwrap().to_string(),
        client_secret: ctx.secret("TWITCH_SECRET").unwrap().to_string(),
        redirect_url: ctx.var("TWITCH_REDIRECT_URL").unwrap().to_string(),
        scope: SCOPES.to_string(),
        auth_url: TWITCH_AUTH_URL.to_string(),
        token_url: TWITCH_TOKEN_URL.to_string(),
        ..Default::default()
    };

    if legacy {
        config.redirect_url = ctx.var("TWITCH_LEGACY_REDIRECT_URL").unwrap().to_string();
    }

    config
}
