use std::collections::HashMap;

use rand::distributions::Alphanumeric;
use rand::Rng;
use worker::wasm_bindgen::JsValue;
use worker::{FormData, FormEntry, Method, Response, Result};

#[derive(Default)]
pub struct OAuthConfig {
    pub name: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_url: String,
    pub scope: String,
    pub auth_url: String,
    pub token_url: String,
    pub extra_params: HashMap<String, String>,
}

pub fn get_redirect_url(config: OAuthConfig) -> String {
    let mut params = vec![
        ("client_id", config.client_id),
        ("redirect_uri", config.redirect_url),
        ("response_type", "code".to_string()),
        ("scope", config.scope),
        ("state", generate_state_string()),
    ];

    if config.extra_params.len() > 0 {
        for (key, val) in config.extra_params.iter() {
            params.push((key, val.to_string()));
        }
    }

    format!(
        "{}?{}",
        config.auth_url,
        serde_urlencoded::to_string(params).unwrap()
    )
}

pub async fn get_token(config: OAuthConfig, form_data: FormData) -> Result<Response> {
    let grant_type: String = get_param_val(&form_data, "grant_type").unwrap_or_default();

    let mut post_data = vec![
        ("client_id", config.client_id),
        ("client_secret", config.client_secret),
        ("grant_type", grant_type.to_string()),
    ];

    match grant_type.as_str() {
        "refresh_token" => {
            post_data.push((
                "refresh_token",
                get_param_val(&form_data, "refresh_token").unwrap(),
            ));
        }
        "authorization_code" => {
            post_data.push(("code", get_param_val(&form_data, "code").unwrap()));
            post_data.push(("redirect_uri", config.redirect_url));
        }
        _ => return Response::error(format!("Invalid grant_type '{}'", grant_type), 400),
    }

    let mut headers = worker::Headers::new();
    headers.set("Content-Type", "application/x-www-form-urlencoded")?;

    let mut req_init = worker::RequestInit::new();
    req_init.with_method(Method::Post);
    req_init.with_headers(headers);
    req_init.with_body(Some(JsValue::from_str(
        serde_urlencoded::to_string(post_data).unwrap().as_str(),
    )));

    let req = worker::Request::new_with_init(&config.token_url, &req_init)?;
    let _resp = worker::Fetch::Request(req).send().await;

    if let Err(e) = _resp {
        let resp = Response::from_json(&serde_json::json!({
            "error": "curl_error",  // Legacy error code used in original PHP script
            "error_description": format!("Request failed with {}", e)
        }))?;
        return Ok(resp.with_status(500));
    }

    let mut resp = _resp.unwrap();
    let status = resp.status_code();
    let _json = resp.json::<HashMap<String, serde_json::Value>>().await;

    if _json.is_err() {
        let res = Response::from_json(&serde_json::json!({
            "error": "parse_error",
            "error_description": format!("Bad JSON response from {}", config.name)
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
                    "error_description":
                        format!("Your {} login token is no longer valid. Please try reconnecting your account.",
                            config.name)
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
                "error_description": format!("Received HTTP {} from {}", status, config.name)
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

fn generate_state_string() -> String {
    let rand_string: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    rand_string
}

fn get_param_val(form_data: &FormData, name: &str) -> Option<String> {
    // This is kinda weird, dunno if there's a better way to do this.
    if let Some(value) = form_data.get(name) {
        if let FormEntry::Field(val) = value {
            return Some(val);
        }
    };

    None
}
