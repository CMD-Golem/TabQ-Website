use axum::{
	http::StatusCode,
	response::{IntoResponse,Response}
};
use serde_json;
use reqwest;
use hex;

pub fn generic_unauthorized_error(err: &str) -> Response {
	let body = err.to_string();

	eprintln!("{body}");
	return (StatusCode::INTERNAL_SERVER_ERROR, body).into_response();
}

pub fn map_reqwest_error(err: reqwest::Error, source: &str) -> Response {
	let status = err.status().unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
	let body = err.to_string();

	eprintln!("[{source}] {body}");
	return (status, body).into_response();
}

pub fn map_serde_error(err: serde_json::Error, source: &str) -> Response {
	let body = err.to_string();

	eprintln!("[{source}] {body}");
	return (StatusCode::INTERNAL_SERVER_ERROR, body).into_response();
}

pub fn map_hex_error(err: hex::FromHexError, source: &str) -> Response {
	let body = err.to_string();

	eprintln!("[{source}] {body}");
	return (StatusCode::UNAUTHORIZED, body).into_response();
}