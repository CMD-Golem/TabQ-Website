use axum::{
	body::Bytes,
	extract::{State,DefaultBodyLimit},
	http::StatusCode,
	routing::post,
	Router,
	response::{IntoResponse, Response},
};
use std::{
	env::var,
	fs,
	io::Cursor,
	path::Path,
};
use hmac::{Hmac, Mac};
use zip::ZipArchive;
use hex;
use sha2::Sha256;
use http::HeaderMap;
// use reqwest;

use crate::error;

pub async fn router() -> Router {
	let secret = var("GITHUB_WEBHOOK_SECRET").expect("Missing GITHUB_WEBHOOK_SECRET env var");

	return Router::new()
		.route("/refresh-frontend", post(refresh))
		.layer(DefaultBodyLimit::max(0x8000000))
		.with_state(secret);
}

// hot refresh static files
async fn refresh(State(secret): State<String>, headers: HeaderMap, body: Bytes) -> Result<Response, Response> {
	let signature = headers
		.get("x-hub-signature-256")
		.ok_or_else(|| error::generic_unauthorized_error("Signature required"))?
		.to_str()
		.map_err(|e| error::map_http_error(e))?;

	let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
	mac.update(&body);

	if signature != hex::encode(mac.finalize().into_bytes()) {
		// println!("Signature is correctly detected as wrong")
		return Err(error::generic_unauthorized_error("Signature invalied"));
	}

	// extract zip
	let reader = Cursor::new(body);
	let frontend_dir = Path::new("/app/frontend");
	let tmp_target = Path::new("/tmp/frontend");

	let mut archive = ZipArchive::new(reader).map_err(|e| error::map_zip_error(e))?;
	archive.extract(tmp_target).map_err(|e| error::map_zip_error(e))?;

	fs::remove_dir_all(frontend_dir).map_err(|e| error::map_path_error(e))?;
	fs::rename(tmp_target, frontend_dir).map_err(|e| error::map_path_error(e))?;

	return Ok((StatusCode::OK).into_response());
}