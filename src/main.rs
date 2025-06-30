use axum::{
	http::{HeaderValue, Method, StatusCode, header},
	response::IntoResponse,
	routing::post,
	routing::get,
	Router
};

use std::{
	net::SocketAddr,
};
use tower_http::{
	cors::CorsLayer,
	trace::TraceLayer,
	services::ServeDir,
};

use serde_json;
use tokio;
use reqwest;

mod error;


#[tokio::main]
async fn main() {
	let app = Router::new()
		.route_service("/", ServeDir::new("static"))
		.route("/api/health", get(health))
		.route("/api/1/publications", post(publications))
		.route("/api/1/pages", post(pages))
		.layer(
			CorsLayer::new()
				.allow_origin("127.0.0.1:3000".parse::<HeaderValue>().unwrap())
				.allow_headers([header::CONTENT_TYPE])
				.allow_methods([Method::GET, Method::POST])
		)
		.layer(TraceLayer::new_for_http());

	let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
	// let addr = SocketAddr::from(([0, 0, 0, 0], 3000)); // docker
	let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
	axum::serve(listener, app).await.unwrap();
}

async fn publications(body: String) -> impl IntoResponse {
	let json_body: serde_json::Value = serde_json::from_str(&body).expect("JSON is broken");
	let date = json_body["date"].as_str().unwrap_or("");
	let amount = json_body["amount"].as_u64().unwrap_or(30);

	let client = reqwest::Client::new();
	let body = client.post("https://epaper.coopzeitung.ch/epaper/1.0/findEditionsFromDateWithInlays")
		.body(format!("{{\"editions\": [{{\"defId\": 1134,\"publicationDate\": \"{date}\"}}],\"maxHits\": {amount},\"startDate\": \"{date}\"}}"))
		.send().await;
	
	println!("{:?}", body);

	return match body {
		Ok(res) => (StatusCode::OK, res.text().await.unwrap_or("Can't read response".to_string())),
		Err(e) => (StatusCode::NOT_FOUND, e.to_string()),
	};
}

async fn pages(body: String) -> Result<impl IntoResponse, (StatusCode, String)> {
	let request: serde_json::Value = serde_json::from_str(&body).map_err(error::map_serde_error)?;
	let date = request["date"].as_str().unwrap_or("");

	let client = reqwest::Client::new();
	let fetch = client.post("https://epaper.coopzeitung.ch/epaper/1.0/getPages")
		.body(format!("{{\"screenInfo\":{{\"width\":1155,\"height\":1060}},\"editions\":[{{\"defId\":1134,\"publicationDate\":\"{date}\"}}]}}"))
		.send().await.map_err(error::map_reqwest_error)?.text().await.map_err(error::map_reqwest_error)?;

	let empty = vec![];
	let json_obj: serde_json::Value = serde_json::from_str(&fetch).map_err(error::map_serde_error)?;
	let pages = json_obj["data"]["pages"].as_array().unwrap_or(&empty);
	let mut images = vec![];

	for page in pages.iter() {
		let image = page["pageDocUrl"]["PREVIEW"]["url"].as_str().unwrap_or("");
		images.push(image.to_string());
	}

	return Ok((StatusCode::OK, body));
}

async fn health() -> StatusCode {
	return StatusCode::OK;
}

