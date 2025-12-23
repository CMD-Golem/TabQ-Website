use axum::{
	http::StatusCode,
	response::{IntoResponse, Response},
	routing::post,
	Router
};
use serde::Serialize;
use http::HeaderMap;
use serde_json;
use reqwest;

use crate::error;

#[derive(Serialize)]
struct Magazines {
	edition_number: u64,
	edition_volume: u64,
	publication_date: String,
}

pub async fn router() -> Router {
	return Router::new()
		.route("/publications", post(publications))
		.route("/pages", post(pages));
}

async fn publications(headers: HeaderMap, body: String) -> Result<Response, Response> {
	let json_body: serde_json::Value = serde_json::from_str(&body).map_err(|e| error::map_serde_error(e, "Magazines"))?;
	let date = json_body["date"].as_str().unwrap_or("");
	let amount = json_body["amount"].as_u64().unwrap_or(5);

	println!("[Magazines] {} fetched publications", headers.get("X-Forwarded-For").and_then(|value| value.to_str().ok()).unwrap_or("Unknow client"));

	let client = reqwest::Client::new();
	let fetch = client.post("https://epaper.coopzeitung.ch/epaper/1.0/findEditionsFromDateWithInlays")
		.body(format!("{{\"editions\": [{{\"defId\": 1134,\"publicationDate\": \"{date}\"}}],\"maxHits\": {amount},\"startDate\": \"{date}\"}}"))
		.send().await.map_err(|e| error::map_reqwest_error(e, "Magazines"))?
		.text().await.map_err(|e| error::map_reqwest_error(e, "Magazines"))?;

	let empty = vec![];
	let json_obj: serde_json::Value = serde_json::from_str(&fetch).map_err(|e| error::map_serde_error(e, "Magazines"))?;
	let pages = json_obj["data"].as_array().unwrap_or(&empty);
	let mut response = vec![];

	for page in pages.iter() {
		let number = page["pages"][0]["editionNumber"].as_u64().unwrap_or(0);
		let volume = page["pages"][0]["editionVolume"].as_u64().unwrap_or(0);
		let date = page["pages"][0]["publicationDate"].as_str().unwrap_or("");
		let obj = Magazines {edition_number: number, edition_volume: volume, publication_date: date.to_string()};
		response.push(obj);
	}

	let response_string = serde_json::to_string(&response).map_err(|e| error::map_serde_error(e, "Magazines"))?;
	
	return Ok((StatusCode::OK, response_string).into_response());

}

async fn pages(headers: HeaderMap, body: String) -> Result<Response, Response> {
	let request: serde_json::Value = serde_json::from_str(&body).map_err(|e| error::map_serde_error(e, "Magazines"))?;
	let date = request["date"].as_str().unwrap_or("");

	println!("[Magazines] {} fetched pages", headers.get("X-Forwarded-For").and_then(|value| value.to_str().ok()).unwrap_or("Unknow client"));

	let client = reqwest::Client::new();
	let fetch = client.post("https://epaper.coopzeitung.ch/epaper/1.0/getPages")
		.body(format!("{{\"screenInfo\":{{\"width\":1155,\"height\":1060}},\"editions\":[{{\"defId\":1134,\"publicationDate\":\"{date}\"}}]}}"))
		.send().await.map_err(|e| error::map_reqwest_error(e, "Magazines"))?
		.text().await.map_err(|e| error::map_reqwest_error(e, "Magazines"))?;

	let empty = vec![];
	let json_obj: serde_json::Value = serde_json::from_str(&fetch).map_err(|e| error::map_serde_error(e, "Magazines"))?;
	let pages = json_obj["data"]["pages"].as_array().unwrap_or(&empty);
	let mut images = vec![];

	for page in pages.iter() {
		let image = page["pageDocUrl"]["PREVIEW"]["url"].as_str().unwrap_or("");
		images.push(image.to_string());
	}

	let image_string = serde_json::to_string(&images).map_err(|e| error::map_serde_error(e, "Magazines"))?;

	return Ok((StatusCode::OK, image_string).into_response());
}