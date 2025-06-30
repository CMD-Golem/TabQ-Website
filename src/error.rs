use axum::http::StatusCode;
use serde_json;
use reqwest;

pub fn map_reqwest_error(err: reqwest::Error) -> (StatusCode, String) {
	println!("Reqwest err");
	let status = err.status().unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
	return (status, format!("{}, Reqwest er", err.to_string()));
}

pub fn map_serde_error(err: serde_json::Error) -> (StatusCode, String) {
	println!("Serde err");
	return (StatusCode::INTERNAL_SERVER_ERROR, format!("{}, Serde er", err.to_string()));
}