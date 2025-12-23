use axum::{
	http::StatusCode,
	response::{IntoResponse, Response},
	routing::{post, delete},
	Router
};
use http::HeaderMap;
use serde_json;
use reqwest;

use crate::error;

pub async fn router() -> Router {
	return Router::new()
		.route("/", post(create))
		.route("/", delete(remove));
}

async fn create(headers: HeaderMap, body: String) -> Result<Response, Response> {
	let json_body: serde_json::Value = serde_json::from_str(&body).map_err(|e| error::map_serde_error(e, "Infomaniak Mail"))?;
	let mailbox_name = json_body["mailbox_name"].as_str().unwrap_or("");
	let mail_hosting_id = json_body["mail_hosting_id"].as_i64().unwrap_or(0);

	println!("{} fetched publications", headers.get("X-Forwarded-For").and_then(|value| value.to_str().ok()).unwrap_or("Unknow client"));

	if mailbox_name.len() > 64 || mailbox_name.len() == 0 {
		return Err(error::generic_unauthorized_error("mailbox_name is invalid"));
	}
	if mail_hosting_id == 0 {
		return Err(error::generic_unauthorized_error("mail_hosting_id is invalid"));
	}

	let client = reqwest::Client::new();
	let fetch = client.post(format!("https://api.infomaniak.com/1/mail_hostings/{mail_hosting_id}/mailboxes"))
		.body(format!("{{\"mailbox_name\": \"{mailbox_name}\", \"target\": \"current_user\", \"link_to_current_user\": true}}"))
		.send().await.map_err(|e| error::map_reqwest_error(e, "Infomaniak Mail"))?
		.text().await.map_err(|e| error::map_reqwest_error(e, "Infomaniak Mail"))?;
	
	return Ok((StatusCode::OK, fetch).into_response());
}

async fn remove(headers: HeaderMap, body: String) -> Result<Response, Response> {
	let json_body: serde_json::Value = serde_json::from_str(&body).map_err(|e| error::map_serde_error(e, "Infomaniak Mail"))?;
	let mailbox_name = json_body["mailbox_name"].as_str().unwrap_or("");
	let mail_hosting_id = json_body["mail_hosting_id"].as_str().unwrap_or("");

	println!("{} fetched publications", headers.get("X-Forwarded-For").and_then(|value| value.to_str().ok()).unwrap_or("Unknow client"));

	if mailbox_name.len() > 64 {
		return Err(error::generic_unauthorized_error("mailbox_name too long"));
	}

	let client = reqwest::Client::new();
	let fetch = client.post(format!("https://api.infomaniak.com/1/mail_hostings/{mail_hosting_id}/mailboxes"))
		.body(format!("{{\"mailbox_name\": \"{mailbox_name}\"}}"))
		.send().await.map_err(|e| error::map_reqwest_error(e, "Infomaniak Mail"))?
		.text().await.map_err(|e| error::map_reqwest_error(e, "Infomaniak Mail"))?;
	
	return Ok((StatusCode::OK, fetch).into_response());
}