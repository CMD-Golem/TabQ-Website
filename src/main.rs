use axum::{
	http::StatusCode,
	response::IntoResponse,
	routing::post,
	routing::get,
	Router
};
use serde::Serialize;
use http::HeaderMap;
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use serde_json;
use tokio;
use reqwest;

mod error;

#[derive(Serialize)]
struct Magazines {
	edition_number: u64,
	edition_volume: u64,
	publication_date: String,
}

#[tokio::main]
async fn main() {
	let app = Router::new()
		.route("/api/health", get(health))
		.route("/api/1/publications", post(publications))
		.route("/api/1/pages", post(pages))
		.fallback_service(ServeDir::new("static"));

	let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
	let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
	axum::serve(listener, app).await.unwrap();
}

async fn publications(headers: HeaderMap, body: String) -> Result<impl IntoResponse, error::AppError> {
	let json_body: serde_json::Value = serde_json::from_str(&body).map_err(error::map_serde_error)?;
	let date = json_body["date"].as_str().unwrap_or("");
	let amount = json_body["amount"].as_u64().unwrap_or(5);

	println!("{} fetched publications", headers.get("X-Forwarded-For").and_then(|value| value.to_str().ok()).unwrap_or("Unknow client"));

	let client = reqwest::Client::new();
	let fetch = client.post("https://epaper.coopzeitung.ch/epaper/1.0/findEditionsFromDateWithInlays")
		.body(format!("{{\"editions\": [{{\"defId\": 1134,\"publicationDate\": \"{date}\"}}],\"maxHits\": {amount},\"startDate\": \"{date}\"}}"))
		.send().await.map_err(error::map_reqwest_error)?.text().await.map_err(error::map_reqwest_error)?;

	let empty = vec![];
	let json_obj: serde_json::Value = serde_json::from_str(&fetch).map_err(error::map_serde_error)?;
	let pages = json_obj["data"].as_array().unwrap_or(&empty);
	let mut response = vec![];

	for page in pages.iter() {
		let number = page["pages"][0]["editionNumber"].as_u64().unwrap_or(0);
		let volume = page["pages"][0]["editionVolume"].as_u64().unwrap_or(0);
		let date = page["pages"][0]["publicationDate"].as_str().unwrap_or("");
		let obj = Magazines {edition_number: number, edition_volume: volume, publication_date: date.to_string()};
		response.push(obj);
	}

	let response_string = serde_json::to_string(&response).map_err(error::map_serde_error)?;
	
	return Ok((StatusCode::OK, response_string));

}

async fn pages(headers: HeaderMap, body: String) -> Result<impl IntoResponse, error::AppError> {
	let request: serde_json::Value = serde_json::from_str(&body).map_err(error::map_serde_error)?;
	let date = request["date"].as_str().unwrap_or("");

	println!("{} fetched pages", headers.get("X-Forwarded-For").and_then(|value| value.to_str().ok()).unwrap_or("Unknow client"));

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

	let image_string = serde_json::to_string(&images).map_err(error::map_serde_error)?;

	return Ok((StatusCode::OK, image_string));
}

async fn health() -> StatusCode {
	return StatusCode::OK;
}