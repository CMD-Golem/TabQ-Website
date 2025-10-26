use axum::{
	http::StatusCode,
	response::{IntoResponse,Response}
};
use serde_json;
use reqwest;
use http;
use zip;

pub fn generic_unauthorized_error(err: &str) -> Response {
	let body = err.to_string();

	println!("{body}");
	return (StatusCode::INTERNAL_SERVER_ERROR, body).into_response();
}

pub fn map_http_error(err: http::header::ToStrError) -> Response {
	let body = err.to_string();

	println!("{body}");
	return (StatusCode::UNAUTHORIZED, body).into_response();
}

pub fn map_reqwest_error(err: reqwest::Error) -> Response {
	let status = err.status().unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
	let body = err.to_string();

	println!("{body}");
	return (status, body).into_response();
}

pub fn map_serde_error(err: serde_json::Error) -> Response {
	let body = err.to_string();

	println!("{body}");
	return (StatusCode::INTERNAL_SERVER_ERROR, body).into_response();
}

pub fn map_zip_error(err: zip::result::ZipError) -> Response {
	let body = err.to_string();

	println!("{body}");
	return (StatusCode::INTERNAL_SERVER_ERROR, body).into_response();
}

pub fn map_path_error(err: std::io::Error) -> Response {
	let body = err.to_string();

	println!("{body}");
	return (StatusCode::INTERNAL_SERVER_ERROR, body).into_response();
}