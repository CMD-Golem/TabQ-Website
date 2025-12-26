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
use tokio::{fs, io::AsyncWriteExt};
use std::{
	collections::{HashMap, HashSet},
	env::var,
	path::Path
};

use crate::error;

#[derive(Clone)]
struct EnvData {
	secret: String,
	temp_dir: String,
	prod_dir: String,
	git_ref: String,
	branch: String,
	repo_map: HashMap<String, String>,
	local_map: HashMap<String, String>,
}

pub fn router() -> Router {
	// Generate repo map from env (determine if a file has to be considered)
	let mut repo_map = HashMap::new();

	if let Ok(val) = var("REPO_MAP") {
		for entry in val.split("|") {
			if let Some((key, value)) = entry.split_once(";") {
				repo_map.insert(key.to_string(), value.to_string());
			}
		}
	}

	// Generate folder map from env (determine where the files have to be stored)
	let mut local_map = HashMap::new();

	if let Ok(val) = var("LOCAL_MAP") {
		for entry in val.split("|") {
			if let Some((key, value)) = entry.split_once(";") {
				local_map.insert(key.to_string(), value.to_string());
			}
		}
	}

	let branch = var("GITHUB_BRANCH").expect("[Workflow] Missing GITHUB_BRANCH env var");
	let env_data = EnvData {
		secret: var("GITHUB_WEBHOOK_SECRET").expect("[Workflow] Missing GITHUB_WEBHOOK_SECRET env var"),
		temp_dir: var("TEMP_DIR").expect("[Workflow] Missing TEMP_DIR env var"),
		prod_dir: var("PROD_DIR").expect("[Workflow] Missing PROD_DIR env var"),
		git_ref: format!("refs/heads/{}", branch),
		branch: branch,
		repo_map: repo_map,
		local_map: local_map,
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
		.ok_or_else(|| error::generic_unauthorized_error("[Workflow1] Signature required"))?
		.as_bytes()
		.strip_prefix(b"sha256=")
		.ok_or_else(|| error::generic_unauthorized_error("[Workflow2] Signature required"))?;

	let signature_bytes = hex::decode(header_signature)
		.map_err(|e| error::map_hex_error(e, "Workflow3"))?;

	let mut mac = Hmac::<Sha256>::new_from_slice(env_data.secret.as_bytes())
		.map_err(|_e| error::generic_unauthorized_error("[Workflow4] Invalid signature length"))?;
	mac.update(body.as_bytes());

	if mac.verify_slice(&signature_bytes).is_err() {
		return Err(error::generic_unauthorized_error("[Workflow5] Signature invalied"));
	}

	// check if push was to the correct branch
	let json_obj: serde_json::Value = serde_json::from_str(&body).map_err(|e| error::map_serde_error(e, "Workflow6"))?;
	match json_obj["ref"].as_str() {
		Some(git_ref) if git_ref == env_data.git_ref => (),
		Some(_) => return Ok((StatusCode::OK, "Push to wrong branch").into_response()),
		None => return Err((StatusCode::BAD_REQUEST, "No ref in push").into_response()),
	};

	// read data from body 
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

	let mut added_files: HashSet<String> = HashSet::new();
	let mut modified_files: HashSet<String> = HashSet::new();
	let mut removed_files: HashSet<String> = HashSet::new();
	
	for commit in commits {
		let id = commit["id"].as_str().unwrap_or_default();
		let message = commit["message"].as_str().unwrap_or_default();
		println!("[Workflow7] Loading commit {message}, ID: {id}");

		create_hashset(commit, "added", &mut added_files, frontend_folder);
		create_hashset(commit, "modified", &mut modified_files, frontend_folder);
		create_hashset(commit, "removed", &mut removed_files, frontend_folder);
	}

	// download new and changed files
	let temp_dir = Path::new(&env_data.temp_dir);
	let client = reqwest::Client::new();
	let repo_url = format!("https://raw.githubusercontent.com/{repo_name}/{}/", env_data.branch);

	added_files.extend(modified_files.iter().cloned());

	'file_loop: for file in &added_files {
		let mut stream = match client.get(format!("{repo_url}{file}")).send().await {
			Ok(res) if res.status().is_success() => res,
			Ok(_) => {
				eprintln!("[Workflow8] Couldnt donwload {file}");
				continue 'file_loop;
			}
			Err(e) => {
				eprintln!("[Workflow9] {file} {e}");
				continue 'file_loop;
			}
		}.bytes_stream();

		// create parent folders
		let path = temp_dir.join(&file);
		let parent_folder = path.parent().unwrap_or(temp_dir);

		match fs::create_dir_all(parent_folder).await {
			Ok(_) => (),
			Err(e) => {
				eprintln!("[Workflow10] {file} {e}");
				continue 'file_loop;
			}
		};

		// write stream to file
		let mut dest = match fs::File::create(&path).await {
			Ok(file) => file,
			Err(e) => {
				eprintln!("[Workflow11] {file} {e}");
				continue 'file_loop;
			}
		};

		while let Some(chunk) = stream.next().await {
			let chunk = match chunk {
				Ok(chunk) => chunk,
				Err(e) => {
					eprintln!("[Workflow12] {file} {e}");
					fs::remove_file(&path).await.unwrap_or_default();
					continue 'file_loop;
				}
			};
			match dest.write_all(&chunk).await {
				Ok(file) => file,
				Err(e) => {
					eprintln!("[Workflow13] {file} {e}");
					fs::remove_file(&path).await.unwrap_or_default();
					continue 'file_loop;
				}
			};
		}
	}


	// remove file from prod
	let prod_dir = match env_data.local_map.get(repo_name) {
		Some(folder) => Path::new(&env_data.prod_dir).join(folder),
		None => return Err((StatusCode::BAD_REQUEST, "Repository isnt in env").into_response()),
	};

	removed_files.extend(modified_files);

	for file in removed_files {
		let prod_path = prod_dir.join(&file.replace(frontend_folder, ""));

		match fs::remove_file(&prod_path).await {
			Ok(_) => (),
			Err(e) => eprintln!("[Workflow14] {file} {e}"),
		}
	}

	// move temp to prod
	for file in added_files {
		let temp_path = temp_dir.join(&file);
		let prod_path = prod_dir.join(&file.replace(frontend_folder, ""));

		// create parent folders
		let parent_folder = prod_path.parent().unwrap_or(&prod_dir);
		match fs::create_dir_all(parent_folder).await {
			Ok(_) => (),
			Err(e) => {
				eprintln!("[Workflow15] {file} {e}");
				continue;
			}
		};

		// move file
		match fs::rename(&temp_path, &prod_path).await {
			Ok(_) => (),
			Err(e) => eprintln!("[Workflow16] {file} {e}"),
		}
	}

	println!("[Workflow17] Finished update");

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