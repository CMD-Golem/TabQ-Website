use axum::{
	http::StatusCode,
	response::{IntoResponse,Response}
};
use serde_json;
use reqwest;

pub struct AppError {
	pub status: StatusCode,
	pub body: String
}

impl IntoResponse for AppError {
	fn into_response(self) -> Response {
		(self.status, self.body).into_response()
	}
}

pub fn map_reqwest_error(err: reqwest::Error) -> AppError {
	println!("Reqwest err");
	let status = err.status().unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
	let body = format!("{}, Reqwest er", err.to_string());

	return AppError {
		status: status,
		body: body
	};
}

pub fn map_serde_error(err: serde_json::Error) -> AppError {
	println!("Serde err");
	let body = format!("{}, Serde er", err.to_string());

	return AppError {
		status: StatusCode::INTERNAL_SERVER_ERROR,
		body: body
	};
}