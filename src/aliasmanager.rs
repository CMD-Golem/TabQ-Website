use axum::{
	http::StatusCode,
	response::{IntoResponse, Response},
	routing::{get, post, patch, delete},
	extract::{State, Path},
	Router
};
use tower_cookies::{
	cookie::{SameSite, time::Duration},
	Cookie,
	Cookies,
	CookieManagerLayer,
	Key
};
use base64::{engine::general_purpose::STANDARD, Engine};
use reqwest::{self, header::CONTENT_TYPE};
use serde::{Serialize, Deserialize};
use std::env::var;
use serde_json;

use crate::ReqwestConfig;
use crate::error;

#[derive(Serialize, Deserialize)]
struct CookieData {
	filter_mailbox: String,
	alias_mailbox: String,
	mail_hosting_id: String,
	bearer: String,
	alias_domain: String,
}

impl CookieData {
	fn get(key_bytes: Vec<u8>, cookies: &Cookies) -> Result<CookieData, Response> {
		let key = Key::try_from(key_bytes.as_slice()).map_err(|e| error::map_cookie_error(e, "Alias Manager"))?;

		let Some(cookie) = cookies.private(&key).get("data") else {
			return Err((StatusCode::INTERNAL_SERVER_ERROR, "[Alias Manager] Cookie can not be decrypted").into_response());
		};

		let data: CookieData = serde_json::from_str(cookie.value()).map_err(|e| error::map_serde_error(e, "Alias Manager"))?;
		return Ok(data);
	}
}

#[derive(Clone)]
struct AppState {
	key_bytes: Vec<u8>,
	reqwest: reqwest::Client,
}

pub async fn router(app: ReqwestConfig) -> Router {
	let key_string = var("ALIAS_MANAGER_KEY").expect("[Alias Manager] Missing ALIAS_MANAGER_KEY env var");
	let key_bytes = STANDARD.decode(key_string).expect("[Alias Manager] Broken ALIAS_MANAGER_KEY env var");

	let state = AppState {
		key_bytes,
		reqwest: app.reqwest
	};

	return Router::new()
		.route("/data", get(get_cookie_data))
		.route("/data", post(update_cookie_data))
		.route("/alias", get(get_alias))
		.route("/alias", post(create_alias))
		.route("/alias/{alias}", delete(remove_alias))
		.route("/filter", get(get_filter))
		.route("/filter", patch(update_filter))
		.with_state(state)
		.layer(CookieManagerLayer::new());
}

async fn get_cookie_data(State(state): State<AppState>, cookies: Cookies) -> Result<Response, Response> {
	let mut data = CookieData::get(state.key_bytes, &cookies)?;

	if !data.bearer.is_empty() {
		data.bearer = "*".to_string();
	}

	let response_string = serde_json::to_string(&data).map_err(|e| error::map_serde_error(e, "Alias Manager"))?;
	return Ok((StatusCode::OK, response_string).into_response())
}

async fn update_cookie_data(State(state): State<AppState>, cookies: Cookies, body: String) -> Result<Response, Response> {
	let mut new_data: CookieData = serde_json::from_str(&body).map_err(|e| error::map_serde_error(e, "Alias Manager"))?;
	let key = Key::try_from(state.key_bytes.as_slice()).map_err(|e| error::map_cookie_error(e, "Alias Manager"))?;
	let data;

	if new_data.bearer == "*".to_string() {
		new_data.bearer = CookieData::get(state.key_bytes, &cookies)?.bearer;
		data = serde_json::to_string(&new_data).map_err(|e| error::map_serde_error(e, "Alias Manager"))?;
	}
	else {
		data = body;
	}

	let cookie = Cookie::build(("data", data))
		.path("/api/aliasmanager/")
		// .secure(true)
		.http_only(true)
		.same_site(SameSite::Strict)
		.max_age(Duration::weeks(100))
		.build();
	
	return Ok((StatusCode::OK, cookies.private(&key).add(cookie)).into_response());
}

async fn get_alias(State(state): State<AppState>, cookies: Cookies) -> Result<Response, Response> {
	let data = CookieData::get(state.key_bytes, &cookies)?;
	let response = state.reqwest.get(format!("https://api.infomaniak.com/1/mail_hostings/{}/mailboxes/{}/aliases", data.mail_hosting_id, data.alias_mailbox))
		.bearer_auth(data.bearer)
		.send().await.map_err(|e| error::map_reqwest_error(e, "Alias Manager"))?;

	let res_status = response.status();
	let res_text = response.text().await.map_err(|e| error::map_reqwest_error(e, "Alias Manager"))?;
	
	return Ok((res_status, res_text).into_response());
}

async fn create_alias(State(state): State<AppState>, cookies: Cookies, body: String) -> Result<Response, Response> {
	let data = CookieData::get(state.key_bytes, &cookies)?;
	let response = state.reqwest.post(format!("https://api.infomaniak.com/1/mail_hostings/{}/mailboxes/{}/aliases", data.mail_hosting_id, data.alias_mailbox))
		.body(body)
		.bearer_auth(data.bearer)
		.header(CONTENT_TYPE, "application/json")
		.send().await.map_err(|e| error::map_reqwest_error(e, "Alias Manager"))?;

	let res_status = response.status();
	let res_text = response.text().await.map_err(|e| error::map_reqwest_error(e, "Alias Manager"))?;
	
	return Ok((res_status, res_text).into_response());
}

async fn remove_alias(State(state): State<AppState>, Path(alias): Path<String>, cookies: Cookies) -> Result<Response, Response> {
	let data = CookieData::get(state.key_bytes, &cookies)?;
	let response = state.reqwest.delete(format!("https://api.infomaniak.com/1/mail_hostings/{}/mailboxes/{}/aliases/{}", data.mail_hosting_id, data.alias_mailbox, alias))
		.bearer_auth(data.bearer)
		.send().await.map_err(|e| error::map_reqwest_error(e, "Alias Manager"))?;

	let res_status = response.status();
	let res_text = response.text().await.map_err(|e| error::map_reqwest_error(e, "Alias Manager"))?;
	
	return Ok((res_status, res_text).into_response());
}

async fn get_filter(State(state): State<AppState>, cookies: Cookies) -> Result<Response, Response> {
	let data = CookieData::get(state.key_bytes, &cookies)?;
	let response = state.reqwest.get(format!("https://api.infomaniak.com/1/mail_hostings/{}/mailboxes/{}/auth/filters", data.mail_hosting_id, data.filter_mailbox))
		.bearer_auth(data.bearer)
		.send().await.map_err(|e| error::map_reqwest_error(e, "Alias Manager"))?;

	let res_status = response.status();
	let res_text = response.text().await.map_err(|e| error::map_reqwest_error(e, "Alias Manager"))?;
	
	return Ok((res_status, res_text).into_response());
}

async fn update_filter(State(state): State<AppState>, cookies: Cookies, body: String) -> Result<Response, Response> {
	let data = CookieData::get(state.key_bytes, &cookies)?;
	let response = state.reqwest.patch(format!("https://api.infomaniak.com/1/mail_hostings/{}/mailboxes/{}/auth/filters/scripts", data.mail_hosting_id, data.filter_mailbox))
		.body(body)
		.bearer_auth(data.bearer)
		.header(CONTENT_TYPE, "application/json")
		.send().await.map_err(|e| error::map_reqwest_error(e, "Alias Manager"))?;

	let res_status = response.status();
	let res_text = response.text().await.map_err(|e| error::map_reqwest_error(e, "Alias Manager"))?;
	
	return Ok((res_status, res_text).into_response());
}