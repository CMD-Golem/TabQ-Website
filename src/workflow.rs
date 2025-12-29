use axum::{
	Router,
	extract::State,
	response::{IntoResponse, Response},
	routing::{get, post},
};
use futures_util::StreamExt;
use hex;
use hmac::{Hmac, Mac};
use http::{HeaderMap, StatusCode};
use reqwest;
use serde_json;
use sha2::Sha256;
use subtle::ConstantTimeEq;
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
	bearer: String,
	temp_dir: String,
	prod_dir: String,
	github_user_agent: String,
	git_ref: String,
	branch: String,
	repo_map: HashMap<String, String>,
	local_map: HashMap<String, String>,
}

pub async fn router() -> Router {
	// Generate repo map from env (determine if a file has to be considered)
	let mut repo_map = HashMap::new();
	let repo_map_str = var("REPO_MAP").expect("[Workflow] Missing REPO_MAP env var");

	for entry in repo_map_str.split("|") {
		if let Some((key, value)) = entry.split_once(";") {
			repo_map.insert(key.to_string(), value.to_string());
		}
	}

	// Generate folder map from env (determine where the files have to be stored)
	let mut local_map = HashMap::new();
	let local_map_str = var("LOCAL_MAP").expect("[Workflow] Missing LOCAL_MAP env var");
	
	for entry in local_map_str.split("|") {
		if let Some((key, value)) = entry.split_once(";") {
			local_map.insert(key.to_string(), value.to_string());
		}
	}

	// fill env_data struct
	let branch = var("GITHUB_BRANCH").expect("[Workflow] Missing GITHUB_BRANCH env var");

	let env_data = EnvData {
		bearer: var("COMPARE_API_BEARER").expect("[Workflow] Missing COMPARE_API_BEARER env var"),
		secret: var("GITHUB_WEBHOOK_SECRET").expect("[Workflow] Missing GITHUB_WEBHOOK_SECRET env var"),
		temp_dir: var("TEMP_DIR").expect("[Workflow] Missing TEMP_DIR env var"),
		prod_dir: var("PROD_DIR").expect("[Workflow] Missing PROD_DIR env var"),
		github_user_agent: var("GITHUB_USER_AGENT").expect("[Workflow] Missing GITHUB_USER_AGENT env var"),
		git_ref: format!("refs/heads/{}", branch),
		branch: branch,
		repo_map: repo_map,
		local_map: local_map,
	};

	// do auto refresh from compare after restart when env var is set to true
	if let Ok(value) = var("AUTO_FETCH") && value.to_ascii_lowercase() == "true" {
		let _ = refresh_from_compare(&env_data).await;
	}

	// return router
	return Router::new()
		.route("/refresh-from-compare", get(refresh_from_compare_bearer))
		.route("/refresh-from-webhook", post(refresh_from_webhook))
		.with_state(env_data);
}

async fn refresh_from_compare_bearer(State(env_data): State<EnvData>, headers: HeaderMap) -> Result<Response, Response> {
	let header_signature = headers
		.get(axum::http::header::AUTHORIZATION)
		.ok_or_else(|| error::generic_unauthorized_error("[Workflow-c1] Bearer required"))?
		.as_bytes()
		.strip_prefix(b"Bearer ")
		.ok_or_else(|| error::generic_unauthorized_error("[Workflow-c2] Bearer required"))?;

	if header_signature.ct_eq(env_data.bearer.as_bytes()).into() {
		return refresh_from_compare(&env_data).await;
	}
	else {
		return Err(error::generic_unauthorized_error("[Workflow-c3] Bearer invalid"));
	}
}

async fn refresh_from_compare(env_data: &EnvData) -> Result<Response, Response> {
	let client = reqwest::Client::new();

	for (repo_name, frontend_folder) in &env_data.repo_map {
		println!("[Worflow-c4] Loading commits from {repo_name}");

		// get latest tag
		let tag_obj = match fetch_json(format!("https://api.github.com/repos/{repo_name}/tags"), env_data, &client).await {
			Ok(obj) => obj,
			Err(e) => {
				eprintln!("[Workflow-c5-{e}");
				continue;
			}
		};
		let Some(tag_name) = tag_obj[0]["name"].as_str() else {
			eprintln!("[Workflow-c6] Latest tag not found");
			continue;
		};

		// get changed files
		let compare_obj = match fetch_json(format!("https://api.github.com/repos/{repo_name}/compare/{tag_name}...{}", env_data.branch), env_data, &client).await {
			Ok(obj) => obj,
			Err(e) => {
				eprintln!("[Workflow-c7-{e}");
				continue;
			}
		};

		match compare_obj["status"].as_str() {
			Some(status) if status == "ahead" => status,
			Some(status) => {
				println!("[Worflow-c8] No files changed, status: {status}");
				continue;
			},
			None => {
				eprintln!("[Worflow-c9] Compare status was not defined");
				continue;
			}
		};

		// check total commits dont exide maximum
		match compare_obj["total_commits"].as_u64(){
			Some(total) if total <= 250 => total,
			Some(_) => {
				eprintln!("[Worflow-c10] There are more then 250 new commits since the latest tag. Please update manually.");
				continue;
			}
			None => {
				eprintln!("[Worflow-c11] Total commits were not defined");
				continue;
			}
		};

		// fill hashset
		let mut added_files: HashSet<String> = HashSet::new();
		let mut modified_files: HashSet<String> = HashSet::new();
		let mut removed_files: HashSet<String> = HashSet::new();

		if let Some(files) = compare_obj["files"].as_array() {
			for file in files {
				let filename = match file["filename"].as_str() {
					Some(name) if name.starts_with(frontend_folder) => name,
					_ => continue,
				};

				match file["status"].as_str() {
					Some(status) if status == "added" => added_files.insert(filename.to_string()),
					Some(status) if status == "removed" => removed_files.insert(filename.to_string()),
					Some(_) => modified_files.insert(filename.to_string()),
					None => continue,
				};
			}
		}

		let _ = download_files(env_data, modified_files, added_files, removed_files, repo_name, frontend_folder).await;
	}

	return Ok((StatusCode::OK).into_response());
}

async fn fetch_json(url: String, env_data: &EnvData, client: &reqwest::Client) -> Result<serde_json::Value, String> {
	let response = client.request(reqwest::Method::GET, url)
		.header(reqwest::header::ACCEPT, "application/vnd.github+json")
		.header(reqwest::header::USER_AGENT, &env_data.github_user_agent)
		.header("X-GitHub-Api-Version", "2022-11-28")
		.send().await.map_err(|e| format!("f1] {e}"))?
		.text().await.map_err(|e| format!("f2] {e}"))?;

	let obj: serde_json::Value = serde_json::from_str(&response).map_err(|e| format!("f3] {e}"))?;

	return Ok(obj);
}

// trigger refresh via github webhook
async fn refresh_from_webhook(State(env_data): State<EnvData>, headers: HeaderMap, body: String) -> Result<Response, Response> {
	// check signature
	let header_signature = headers
		.get("x-hub-signature-256")
		.ok_or_else(|| error::generic_unauthorized_error("[Workflow-w1] Signature required"))?
		.as_bytes()
		.strip_prefix(b"sha256=")
		.ok_or_else(|| error::generic_unauthorized_error("[Workflow-w2] Signature required"))?;

	let signature_bytes = hex::decode(header_signature)
		.map_err(|e| error::map_hex_error(e, "Workflow-w3"))?;

	let mut mac = Hmac::<Sha256>::new_from_slice(env_data.secret.as_bytes())
		.map_err(|_e| error::generic_unauthorized_error("[Workflow-w4] Invalid signature length"))?;
	mac.update(body.as_bytes());

	if mac.verify_slice(&signature_bytes).is_err() {
		return Err(error::generic_unauthorized_error("[Workflow-w5] Signature invalid"));
	}

	// check if push was to the correct branch
	let json_obj: serde_json::Value = serde_json::from_str(&body).map_err(|e| error::map_serde_error(e, "Workflow-w6"))?;
	match json_obj["ref"].as_str() {
		Some(git_ref) if git_ref == env_data.git_ref => (),
		Some(_) => {
			println!("[Workflow-w7] Push to another branch");
			return Ok((StatusCode::OK, "Push to another branch").into_response());
		},
		None => return Err(error::generic_request_error("[Workflow-w8] No ref in push")),
	};

	// read data from body 
	let Some(commits) = json_obj["commits"].as_array() else {
		println!("[Workflow-w9] No commits in push");
		return Ok((StatusCode::OK, "No commits in push").into_response());
	};

	let Some(repo_name) = json_obj["repository"]["full_name"].as_str() else {
		return Err(error::generic_request_error("[Workflow-w10] Repository name is not defined"));
	};

	let Some(frontend_folder) = env_data.repo_map.get(repo_name) else {
		return Err(error::generic_request_error("[Workflow-w11] Repository is not in repo map"));
	};

	let mut added_files: HashSet<String> = HashSet::new();
	let mut modified_files: HashSet<String> = HashSet::new();
	let mut removed_files: HashSet<String> = HashSet::new();
	
	for commit in commits {
		let id = commit["id"].as_str().unwrap_or_default();
		println!("[Workflow-w12] Loading commit from {repo_name}, ID: {id}");

		create_hashset(commit, "added", &mut added_files, frontend_folder);
		create_hashset(commit, "modified", &mut modified_files, frontend_folder);
		create_hashset(commit, "removed", &mut removed_files, frontend_folder);
	}

	return download_files(&env_data, modified_files, added_files, removed_files, repo_name, frontend_folder).await;
}

fn create_hashset(commit: &serde_json::Value, key: &str, hashset: &mut HashSet<String>, frontend_folder: &str) {
	let Some(files) = commit[key].as_array() else {
		return;
	};
	
	for file in files {
		match file.as_str() {
			Some(file) if file.starts_with(frontend_folder) => {
				hashset.insert(file.to_string());
			},
			_ => continue,
		}
	}
}

async fn download_files(
	env_data: &EnvData,
	modified_files: HashSet<String>,
	mut added_files: HashSet<String>,
	mut removed_files: HashSet<String>,
	repo_name: &str,
	frontend_folder: &str
) -> Result<Response, Response> {
	// download new and changed files
	let temp_dir = Path::new(&env_data.temp_dir);
	let client = reqwest::Client::new();
	let repo_url = format!("https://raw.githubusercontent.com/{repo_name}/{}/", env_data.branch);

	added_files.extend(modified_files.iter().cloned());

	'file_loop: for file in &added_files {
		let mut stream = match client.get(format!("{repo_url}{file}")).send().await {
			Ok(res) if res.status().is_success() => res,
			Ok(_) => {
				eprintln!("[Workflow-d1] Could not donwload {file}");
				continue 'file_loop;
			}
			Err(e) => {
				eprintln!("[Workflow-d2] {file} {e}");
				continue 'file_loop;
			}
		}.bytes_stream();

		// create parent folders
		let path = temp_dir.join(&file);
		let parent_folder = path.parent().unwrap_or(temp_dir);

		match fs::create_dir_all(parent_folder).await {
			Ok(_) => (),
			Err(e) => {
				eprintln!("[Workflow-d3] {file} {e}");
				continue 'file_loop;
			}
		};

		// write stream to file
		let mut dest = match fs::File::create(&path).await {
			Ok(file) => file,
			Err(e) => {
				eprintln!("[Workflow-d4] {file} {e}");
				continue 'file_loop;
			}
		};

		while let Some(chunk) = stream.next().await {
			let chunk = match chunk {
				Ok(chunk) => chunk,
				Err(e) => {
					eprintln!("[Workflow-d5] {file} {e}");
					fs::remove_file(&path).await.unwrap_or_default();
					continue 'file_loop;
				}
			};
			match dest.write_all(&chunk).await {
				Ok(file) => file,
				Err(e) => {
					eprintln!("[Workflow-d6] {file} {e}");
					fs::remove_file(&path).await.unwrap_or_default();
					continue 'file_loop;
				}
			};
		}
	}


	// remove file from prod
	let mut count_removed = 0;
	let prod_dir = match env_data.local_map.get(repo_name) {
		Some(folder) => Path::new(&env_data.prod_dir).join(folder),
		None => return Err(error::generic_request_error("[Workflow-d7] Repository is not in local map")),
	};

	removed_files.extend(modified_files);

	for file in removed_files {
		let prod_path = prod_dir.join(&file.replace(frontend_folder, ""));

		match fs::remove_file(&prod_path).await {
			Ok(_) => count_removed += 1,
			Err(e) => eprintln!("[Workflow-d8] {file} {e}"),
		}
	}

	// move temp to prod
	let mut count_added = 0;

	for file in added_files {
		let temp_path = temp_dir.join(&file);
		let prod_path = prod_dir.join(&file.replace(frontend_folder, ""));

		// create parent folders
		let parent_folder = prod_path.parent().unwrap_or(&prod_dir);
		match fs::create_dir_all(parent_folder).await {
			Ok(_) => count_added += 1,
			Err(e) => {
				eprintln!("[Workflow-d9] {file} {e}");
				continue;
			}
		};

		// move file
		match fs::rename(&temp_path, &prod_path).await {
			Ok(_) => (),
			Err(e) => eprintln!("[Workflow-d10] {file} {e}"),
		}
	}

	println!("[Workflow-d11] Finished update with {count_added} added/modified and {count_removed} removed files");

	return Ok((StatusCode::OK).into_response());
}