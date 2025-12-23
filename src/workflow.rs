use axum::{
	response::{IntoResponse, Response},
	extract::State,
	routing::post,
	Router,
};
use std::{
	collections::HashSet,
	env::var,
	path::Path,
	fs,
};
use hmac::{Hmac, Mac};
use http::{HeaderMap, StatusCode};
use tokio::{fs::File, io::AsyncWriteExt};
use reqwest;
use serde_json;
use sha2::Sha256;
use futures_util::StreamExt;

use crate::error;

// Defintions
const TEMP_DIR: &str = "/tmp-static/";
const PROD_DIR: &str = "/static/";
const BRANCH: &str = "main";

pub async fn router() -> Router {
	let signature = var("GITHUB_WEBHOOK_SIGNATURE").expect("[Workflow] Missing GITHUB_WEBHOOK_SIGNATURE env var");

	return Router::new()
		.route("/refresh-frontend", post(refresh))
		.with_state(signature);
}

// hot refresh static files
async fn refresh(State(server_signature): State<String>, headers: HeaderMap, body: String) -> Result<Response, Response> {
	// check signature
	let header_signature = headers
		.get("x-hub-signature-256")
		.ok_or(error::generic_unauthorized_error("[Workflow] Signature required"))?
		.to_str()
		.map_err(|e| error::map_http_error(e, "Workflow"))?;

	let mut mac = Hmac::<Sha256>::new_from_slice(header_signature.as_bytes())
		.map_err(|_e| error::generic_unauthorized_error("[Workflow] Invalid signature length"))?;
	mac.update(body.as_bytes());

	if mac.verify_slice(server_signature.as_bytes()).is_err() {
		return Err(error::generic_unauthorized_error("[Workflow] Signature invalied"));
	}

	// read data from body
	let json_obj: serde_json::Value = serde_json::from_str(&body).map_err(|e| error::map_serde_error(e, "Workflow"))?;
	let commits = match json_obj["commits"].as_array() {
		Some(commits) => commits,
		None => return Ok((StatusCode::OK, "No commits in push").into_response()),
	};

	let repo_url = match json_obj["repository"]["full_name"].as_str() {
		Some(name) => format!("https://api.github.com/repos/{name}contents/"),
		None => return Err((StatusCode::BAD_REQUEST, "Repository isnt defined").into_response()),
	};

	let mut create_file: HashSet<String> = HashSet::new();
	let mut move_files: HashSet<String> = HashSet::new();
	
	for commit in commits {
		let id = commit["id"].as_str().unwrap_or_default();
		let message = commit["message"].as_str().unwrap_or_default();
		println!("[Workflow] Loading commit {message}, ID: {id}");

		create_hashset(commit, "added", &mut create_file);
		create_hashset(commit, "modified", &mut create_file);
		create_hashset(commit, "removed", &mut move_files);
	}

	// dowload new and changed files
	let temp_dir = Path::new(TEMP_DIR);
	let client = reqwest::Client::new();

	'file_loop: for file in &create_file {
		let mut stream = match client.get(format!("{repo_url}{file}?ref={BRANCH}")).send().await {
			Ok(res) if res.status().is_success() => res,
			Ok(_) => {
				eprintln!("[Workflow] Couldnt donwload {file}");
				continue 'file_loop;
			}
			Err(e) => {
				eprintln!("[Workflow] {e}");
				continue 'file_loop;
			}
		}.bytes_stream();

		// write stream to file
		let path = temp_dir.join(&file);
		let mut dest = match File::create(&path).await {
			Ok(file) => file,
			Err(e) => {
				eprintln!("[Workflow] {e}");
				continue 'file_loop;
			}
		};

		while let Some(chunk) = stream.next().await {
			let chunk = match chunk {
				Ok(chunk) => chunk,
				Err(e) => {
					eprintln!("[Workflow] {e}");
					fs::remove_file(&path).unwrap_or_default();
					continue 'file_loop;
				}
			};
			match dest.write_all(&chunk).await {
				Ok(file) => file,
				Err(e) => {
					eprintln!("[Workflow] {e}");
					fs::remove_file(&path).unwrap_or_default();
					continue 'file_loop;
				}
			};
		}
	}

	// move temp to prod
	let prod_dir = Path::new(PROD_DIR);
	move_files.extend(create_file);

	for file in move_files {
		let temp_path = temp_dir.join(&file);
		let prod_path = prod_dir.join(&file);

		// remove file from prod if it exists
		if prod_path.exists() {
			match fs::remove_file(&prod_path) {
				Ok(_) => (),
				Err(e) => eprintln!("[Workflow] {e}"),
			}
		}

		// move file to prod if it exists in temp
		if temp_path.exists() {
			match fs::rename(temp_path, prod_path) {
				Ok(_) => (),
				Err(e) => eprintln!("[Workflow] {e}"),
			}
		}
	}

	println!("[Workflow] Finished update");

	return Ok((StatusCode::OK).into_response());
}

fn create_hashset(commit: &serde_json::Value, key: &str, hashset: &mut HashSet<String>) {
	 if let Some(files) = commit[key].as_array() {
		for file in files.iter().filter_map(|v| v.as_str()) {
			hashset.insert(file.to_string());
		}
	}
}