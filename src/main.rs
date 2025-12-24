use axum::{
	http::{StatusCode, Request},
	routing::{get, any},
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
// mod infomaniakmail;


#[tokio::main]
async fn main() {
	let api = Router::new()
		// .nest("/infomaniakmail", infomaniakmail::router().await)
		.nest("/magazines", magazines::router())
		.nest("/workflow", workflow::router())
		.route("/health", get(health))
		.route("/test", any(test));

	let frontend = Router::new()
		.fallback_service(ServeDir::new("static").not_found_service(ServeFile::new("static/404.html")))
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
		println!("[Static] {path} {referrer}");
	}
	else {
		eprintln!("[Static] Failed to serve {path} {client} {referrer}");
	}

	return response;
}

async fn health() -> StatusCode {
	return StatusCode::OK;
}

async fn test(headers: http::HeaderMap, body: String) -> StatusCode {
	println!("[Test] Headers: {:?}", headers);
	println!("[Test] Body: {body}");
	return StatusCode::OK;
}