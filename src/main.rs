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

mod magazines;
mod workflow;
mod aliasmanager;
mod error;

#[derive(Clone)]
pub struct ReqwestConfig {
	pub reqwest: reqwest::Client,
}

impl ReqwestConfig {
	pub fn new() -> Self {
		Self {
			reqwest: Self::build(),
		}
	}

	fn build() -> reqwest::Client {
		// https://github.com/vercel/next.js/pull/88869/changes
		let mut builder = reqwest::Client::builder();

		builder = builder.tls_certs_merge(webpki_root_certs::TLS_SERVER_ROOT_CERTS.iter().map(
			|der| {
				reqwest::Certificate::from_der(der)
					.expect("webpki_root_certs should parse correctly")
			}),
		);
		builder.build().expect("failed to create HTTP client")
	}
}


#[tokio::main]
async fn main() {
	let client = ReqwestConfig::new();

	let api = Router::new()
		.nest("/magazines", magazines::router(client.clone()))
		.nest("/workflow", workflow::router(client.clone()).await)
		.nest("/aliasmanager", aliasmanager::router(client.clone()).await)
		.route("/health", get(health))
		.route("/test", any(test));

	let startpage = Router::new()
		.fallback_service(ServeDir::new("static/startpage")
		.fallback(ServeFile::new("static/startpage/index.html")));

	let frontend = Router::new()
		.nest("/startpage", startpage)
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