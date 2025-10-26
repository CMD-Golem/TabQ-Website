use axum::{
	http::{StatusCode, Request},
	routing::get,
	middleware,
	Router,
	body::Body,
	response::Response
};
use tower_http::services::{
	ServeDir,
	ServeFile
};
use std;
use tokio;

mod magazines;
mod workflow;
mod error;


#[tokio::main]
async fn main() {
	let api = Router::new()
		.nest("/magazines", magazines::router().await)
		.nest("/workflow", workflow::router().await)
		.route("/health", get(health));

	let frontend = Router::new()
		.fallback_service(
			ServeDir::new("static")
			.not_found_service(ServeFile::new("static/404.html"))
		)
		.layer(middleware::from_fn(log_static));

	let app = Router::new()
		.nest("/api", api)
		.merge(frontend);

	let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 3000));
	let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
	axum::serve(listener, app).await.unwrap();
}

async fn log_static(req: Request<Body>, next: middleware::Next) -> Response {
	let path = req.uri().path().to_string();
	let referrer = req.headers().get("User-Agent").and_then(|value| value.to_str().ok()).unwrap_or("Unknow User-Agent").to_string();
	let client = req.headers().get("X-Forwarded-For").and_then(|value| value.to_str().ok()).unwrap_or("Unknow client").to_string();

	let response = next.run(req).await;

	if ! matches!(
		response.headers().get("content-type").and_then(|v| v.to_str().ok()),
		Some(s) if s.starts_with("text/html")
	) {
		return response;
	}

	if response.status().is_success() {
		println!("{path} {referrer}");
	}
	else {
		println!("Failed to serve {path} {client} {referrer}");
	}

	return response;
}

async fn health() -> StatusCode {
	return StatusCode::OK;
}