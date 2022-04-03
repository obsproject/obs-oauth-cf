use worker::{FormData, Response, Result, RouteContext};

use crate::platforms::oauth;
use crate::platforms::oauth::OAuthConfig;

const SCOPES: &str = "stream.default.read+chat.default.read";
const RESTREAM_AUTH_URL: &str = "https://api.restream.io/login";
const RESTREAM_TOKEN_URL: &str = "https://api.restream.io/oauth/token";

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
        name: "Restream".to_string(),
        client_id: ctx.secret("RESTREAM_ID").unwrap().to_string(),
        client_secret: ctx.secret("RESTREAM_SECRET").unwrap().to_string(),
        redirect_url: ctx.var("RESTREAM_REDIRECT_URL").unwrap().to_string(),
        scope: SCOPES.to_string(),
        auth_url: RESTREAM_AUTH_URL.to_string(),
        token_url: RESTREAM_TOKEN_URL.to_string(),
        ..Default::default()
    };

    if legacy {
        config.redirect_url = ctx.var("RESTREAM_LEGACY_REDIRECT_URL").unwrap().to_string();
    }

    config
        .extra_params
        .insert("include_granted_scopes".to_string(), "true".to_string());

    config
}
