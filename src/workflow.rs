use axum::{
	Router, extract::State, response::{IntoResponse, Response}, routing::post
};
use futures_util::StreamExt;
use hex;
use hmac::{Hmac, Mac};
use http::{HeaderMap, StatusCode};
use reqwest;
use serde_json;
use sha2::Sha256;
use tokio::{fs::File, io::AsyncWriteExt};
use std::{
	collections::{HashMap, HashSet},
	env::var,
	fs,
	path::Path
};

use crate::error;

#[derive(Clone)]
struct EnvData {
	secret: String,
	branch: String,
	temp_dir: String,
	prod_dir: String,
	repo_map: HashMap<String, String>,
}

pub fn router() -> Router {
	let mut repo_map = HashMap::new();

	if let Ok(val) = var("REPO_MAP") {
		for entry in val.split("|") {
			if let Some((key, value)) = entry.split_once(";") {
				repo_map.insert(key.to_string(), value.to_string());
			}
		}
	}

	let env_data = EnvData {
		secret: var("GITHUB_WEBHOOK_SECRET").expect("[Workflow] Missing GITHUB_WEBHOOK_SECRET env var"),
		branch: var("GITHUB_BRANCH").expect("[Workflow] Missing GITHUB_BRANCH env var"),
		temp_dir: var("TEMP_DIR").expect("[Workflow] Missing TEMP_DIR env var"),
		prod_dir: var("PROD_DIR").expect("[Workflow] Missing PROD_DIR env var"),
		repo_map: repo_map,
	};

	return Router::new()
		.route("/refresh-frontend", post(refresh))
		.with_state(env_data);
}

// hot refresh static files
async fn refresh(State(env_data): State<EnvData>, headers: HeaderMap, body: String) -> Result<Response, Response> {
	// check signature
	let header_signature = headers
		.get("x-hub-signature-256")
		.ok_or_else(|| error::generic_unauthorized_error("[Workflow] Signature required"))?
		.as_bytes()
		.strip_prefix(b"sha256=")
		.ok_or_else(|| error::generic_unauthorized_error("[Workflow] Signature required"))?;

	let signature_bytes = hex::decode(header_signature)
		.map_err(|e| error::map_hex_error(e, "Workflow"))?;

	let mut mac = Hmac::<Sha256>::new_from_slice(env_data.secret.as_bytes())
		.map_err(|_e| error::generic_unauthorized_error("[Workflow] Invalid signature length"))?;
	mac.update(body.as_bytes());

	if mac.verify_slice(&signature_bytes).is_err() {
		return Err(error::generic_unauthorized_error("[Workflow] Signature invalied"));
	}

	// read data from body 
	let json_obj: serde_json::Value = serde_json::from_str(&body).map_err(|e| error::map_serde_error(e, "Workflow"))?;
	let commits = match json_obj["commits"].as_array() {
		Some(commits) => commits,
		None => return Ok((StatusCode::OK, "No commits in push").into_response()),
	};

	let repo_name = match json_obj["repository"]["full_name"].as_str() {
		Some(name) => name,
		None => return Err((StatusCode::BAD_REQUEST, "Repository isnt defined").into_response()),
	};

	let frontend_folder = match env_data.repo_map.get(repo_name) {
		Some(folder) => folder,
		None => return Err((StatusCode::BAD_REQUEST, "Repository isnt in env").into_response()),
	};

	let mut create_file: HashSet<String> = HashSet::new();
	let mut move_files: HashSet<String> = HashSet::new();
	
	for commit in commits {
		let id = commit["id"].as_str().unwrap_or_default();
		let message = commit["message"].as_str().unwrap_or_default();
		println!("[Workflow] Loading commit {message}, ID: {id}");

		create_hashset(commit, "added", &mut create_file, frontend_folder);
		create_hashset(commit, "modified", &mut create_file, frontend_folder);
		create_hashset(commit, "removed", &mut move_files, frontend_folder);
	}

	// dowload new and changed files
	let temp_dir = Path::new(&env_data.temp_dir);
	let client = reqwest::Client::new();
	let repo_url = format!("https://api.github.com/repos/{repo_name}contents/");

	'file_loop: for file in &create_file {
		let mut stream = match client.get(format!("{repo_url}{file}?ref={}", env_data.branch)).send().await {
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
	let prod_dir = Path::new(&env_data.prod_dir);
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

fn create_hashset(commit: &serde_json::Value, key: &str, hashset: &mut HashSet<String>, frontend_folder: &str) {
	 if let Some(files) = commit[key].as_array() {
		for file in files.iter().filter_map(|v| v.as_str()) {
			if file.starts_with(frontend_folder) {
				hashset.insert(file.to_string());
			}
		}
	}
}