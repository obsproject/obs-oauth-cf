use std::collections::HashMap;

use serde::Serialize;
use serde_json;
use serde_qs;
use worker::wasm_bindgen::JsValue;
use worker::{FormData, Method, Response, Result, RouteContext};

use crate::platforms::utils::{generate_state_string, get_param_val};

const SCOPES: &str = "channel:read:stream_key";
const TWITCH_AUTH_URL: &str = "https://id.twitch.tv/oauth2/authorize";
const TWITCH_TOKEN_URL: &str = "https://id.twitch.tv/oauth2/token";

#[derive(Serialize)]
struct RedirectParams {
    client_id: String,
    redirect_uri: String,
    response_type: String,
    scope: String,
    state: String,
}

pub fn get_redirect_url(ctx: &RouteContext<()>) -> String {
    let q = RedirectParams {
        client_id: ctx.secret("TWITCH_ID").unwrap().to_string(),
        redirect_uri: ctx.secret("TWITCH_REDIRECT_URL").unwrap().to_string(),
        response_type: "code".to_string(),
        scope: SCOPES.to_string(),
        state: generate_state_string(),
    };

    format!("{}?{}", TWITCH_AUTH_URL, serde_qs::to_string(&q).unwrap())
}

pub async fn get_token(form_data: FormData, ctx: &RouteContext<()>) -> Result<Response> {
    let grant_type: String = get_param_val(&form_data, "grant_type").unwrap_or_default();
    let mut post_data: HashMap<&str, String> = HashMap::from([
        ("client_id", ctx.secret("TWITCH_ID").unwrap().to_string()),
        (
            "client_secret",
            ctx.secret("TWITCH_SECRET").unwrap().to_string(),
        ),
        ("grant_type", grant_type.to_string()),
    ]);

    match grant_type.as_str() {
        "refresh_token" => {
            post_data.insert(
                "refresh_token",
                get_param_val(&form_data, "refresh_token").unwrap(),
            );
        }
        "authorization_code" => {
            post_data.insert("code", get_param_val(&form_data, "code").unwrap());
            post_data.insert(
                "redirect_uri",
                ctx.secret("TWITCH_REDIRECT_URL").unwrap().to_string(),
            );
        }
        _ => return Response::error(format!("Invalid grant_type '{}'", grant_type), 400),
    }

    let mut headers = worker::Headers::new();
    headers.set("Content-Type", "application/x-www-form-urlencoded")?;

    let mut req_init = worker::RequestInit::new();
    req_init.with_method(Method::Post);
    req_init.with_body(Some(JsValue::from_str(
        serde_qs::to_string(&post_data).unwrap().as_str(),
    )));

    let req = worker::Request::new_with_init(TWITCH_TOKEN_URL, &req_init)?;
    let _resp = worker::Fetch::Request(req).send().await;

    if _resp.is_err() {
        let resp = Response::from_json(&serde_json::json!({
            "error": "curl_error",
            "error_description": format!("Request failed with {}", _resp.err().unwrap())
        }))?;
        return Ok(resp.with_status(500));
    }

    let mut resp = _resp.unwrap();
    let status = resp.status_code();
    let _json = resp.json::<HashMap<String, serde_json::Value>>().await;

    if _json.is_err() {
        let res = Response::from_json(&serde_json::json!({
            "error": "parse_error",
            "error_description": "Bad JSON response from Twitch"
        }))?;
        return Ok(res.with_status(500));
    }

    let data = _json.unwrap();

    if status != 200 {
        let resp_data: serde_json::Value;

        if data.contains_key("message") {
            if data["message"].as_str().unwrap() == "Invalid refresh token" {
                resp_data = serde_json::json!({
                    "error": "Error",
                    "error_description": "Your Twitch login token is no longer valid. Please try reconnecting your account."
                });
            } else {
                resp_data = serde_json::json!({
                    "error": "Error",
                    "error_description": data["message"].as_str().unwrap()
                })
            };
        } else {
            resp_data = serde_json::json!({
                "error": "status_error",
                "error_description": format!("Received HTTP {} from Twitch", status)
            });
        }
        let res = Response::from_json(&resp_data)?;
        return Ok(res.with_status(status));
    }

    let res = Response::from_json(&data)?;
    if data.contains_key("error") {
        return Ok(res.with_status(500));
    }

    Ok(res)
}
