use axum::{
	extract::State,
	routing::post,
	Router,
	response::{IntoResponse, Response},
};
use http::{
	HeaderMap,
	header::AUTHORIZATION,
	StatusCode
};
use std::{
	env::var, f64::consts, fs, io::Cursor, path::Path, sync::Arc
};
use reqwest::{self, Client};
use serde_json::{self, Value};
use tokio_stream::StreamExt;
use tokio::{fs::File, io::AsyncWriteExt};

use crate::error;

#[derive(Clone)]
struct AppState {
	secret: String,
	repository: String,
}

pub async fn router() -> Router {
	let state = AppState {
		secret: var("GITHUB_WEBHOOK_SECRET").expect("Missing GITHUB_WEBHOOK_SECRET env var"),
		repository: var("GITHUB_REPOSITORY").expect("Missing GITHUB_REPOSITORY env var"), // https://api.github.com/repos/CMD-Golem/TabQ-Website/contents/static?ref=main
	};

	return Router::new()
		.route("/refresh-frontend", post(refresh))
		.with_state(Arc::new(state));
}

// hot refresh static files
async fn refresh(State(state,): State<Arc<AppState>>, headers: HeaderMap) -> Result<Response, Response> {
	let request_bearer = headers
		.get(AUTHORIZATION)
		.ok_or(error::generic_unauthorized_error("Signature required"))?
		.to_str()
		.map_err(|e| error::map_http_error(e))?
		.strip_prefix("Bearer ")
		.ok_or(error::generic_unauthorized_error("Expected Bearer token"))?;

	if request_bearer != state.secret {
		// println!("Signature is correctly detected as wrong");
		return Err(error::generic_unauthorized_error("Signature invalied"));
	}

	let client = Client::new();
	let main_dir_json = client.get(&state.repository).send().await.map_err(error::map_reqwest_error)?.text().await.map_err(error::map_reqwest_error)?;

	let serde_obj: serde_json::Value = serde_json::from_str(&main_dir_json).map_err(error::map_serde_error)?;
	let array = serde_obj.as_array().ok_or(error::generic_unauthorized_error("Malformed "))?;

	let tmp_target = Path::new("/tmp/frontend");

	for item in array.iter() {
		let item_type = item["type"].as_str().unwrap_or("");

		if item["type"] == "dir" {
			let max;
		}
		else {
			let response = download_file(&item, &tmp_target).await;
		}
	}



	let frontend_dir = Path::new("/app/frontend");


	fs::remove_dir_all(frontend_dir).map_err(|e| error::map_path_error(e))?;
	fs::rename(tmp_target, frontend_dir).map_err(|e| error::map_path_error(e))?;

	return Ok((StatusCode::OK).into_response());
}

async fn download_file(item: &Value, folder_path: &Path) -> Result<(), String> {
	let item_name = item["name"].as_str().ok_or("Could not read name field".to_string())?;
	let item_download = item["download_url"].as_str().ok_or("Could not read name field".to_string())?;
	let mut dest = File::create(folder_path.join(item_name)).await.map_err(|err| format!("{err}"))?;

	let client = Client::new();
	let response = client.get(item_download).send().await.map_err(|err| format!("{err}"))?;

	if !response.status().is_success() {
		return Err("Failed to download file".to_string());
	}

	// Stream the body and write to file
	while let Some(chunk) = response.bytes_stream().next().await {
		let chunk = match chunk {
			Ok(chunk) => chunk,
			Err(e) => {
				println!("{e}");
				break;
			}
		};
		dest.write_all(&chunk).await.map_err(|err| format!("{err}"))?;
	}

	return Ok(());
}