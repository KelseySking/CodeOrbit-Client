use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    net::TcpStream,
    path::PathBuf,
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Manager};

const TARGETS_FILE: &str = "runtime-targets.json";
const KEYRING_SERVICE: &str = "CodeOrbit Client Runtime Target";
const LOCAL_RUNTIME_REPO: &str = r"D:\OtherWork\CodeOrbit-Rust";
const LOCAL_RUNTIME_TOKEN: &str = "dev-token";
const LOCAL_RUNTIME_PORT: u16 = 32145;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RuntimeTarget {
    id: String,
    name: String,
    base_url: String,
    created_at_utc: String,
    updated_at_utc: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RuntimeTargetsFile {
    targets: Vec<RuntimeTarget>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SaveRuntimeTargetRequest {
    id: Option<String>,
    name: String,
    base_url: String,
    token: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LocalRuntimeStartResult {
    success: bool,
    message: String,
}

#[tauri::command]
fn list_runtime_targets(app: AppHandle) -> Result<Vec<RuntimeTarget>, String> {
    Ok(read_targets(&app)?.targets)
}

#[tauri::command]
fn save_runtime_target(
    app: AppHandle,
    request: SaveRuntimeTargetRequest,
) -> Result<RuntimeTarget, String> {
    let name = request.name.trim();
    let base_url = normalize_base_url(&request.base_url)?;
    let token = request.token.as_deref().map(str::trim).unwrap_or("");

    if name.is_empty() {
        return Err("Target name is required.".into());
    }

    let mut file = read_targets(&app)?;
    let now = now_stamp();
    let id = request.id.unwrap_or_else(new_target_id);
    let existing = file.targets.iter().find(|target| target.id == id).cloned();

    if existing.is_none() && token.is_empty() {
        return Err("Token is required for a new target.".into());
    }

    if !token.is_empty() {
        token_entry(&id)?
            .set_password(token)
            .map_err(|err| format!("Could not save token in OS credential storage: {err}"))?;
    }

    let target = RuntimeTarget {
        id: id.clone(),
        name: name.to_string(),
        base_url,
        created_at_utc: existing
            .as_ref()
            .map(|target| target.created_at_utc.clone())
            .unwrap_or_else(|| now.clone()),
        updated_at_utc: now,
    };

    if let Some(index) = file.targets.iter().position(|candidate| candidate.id == id) {
        file.targets[index] = target.clone();
    } else {
        file.targets.push(target.clone());
    }

    file.targets
        .sort_by_key(|target| target.name.to_ascii_lowercase());
    write_targets(&app, &file)?;
    Ok(target)
}

#[tauri::command]
fn delete_runtime_target(app: AppHandle, id: String) -> Result<(), String> {
    let mut file = read_targets(&app)?;
    file.targets.retain(|target| target.id != id);
    write_targets(&app, &file)?;

    let _ = token_entry(&id).and_then(|entry| {
        entry
            .delete_credential()
            .map_err(|err| format!("Could not delete token: {err}"))
    });

    Ok(())
}

#[tauri::command]
fn get_runtime_target_token(id: String) -> Result<String, String> {
    token_entry(&id)?
        .get_password()
        .map_err(|err| format!("Could not load target token from OS credential storage: {err}"))
}

#[tauri::command]
fn start_local_runtime() -> Result<LocalRuntimeStartResult, String> {
    let repo = PathBuf::from(LOCAL_RUNTIME_REPO);
    if !repo.join("Cargo.toml").exists() {
        return Err(format!(
            "CodeOrbit Rust repo was not found at {LOCAL_RUNTIME_REPO}."
        ));
    }

    if TcpStream::connect(("127.0.0.1", LOCAL_RUNTIME_PORT)).is_ok() {
        return Ok(LocalRuntimeStartResult {
            success: true,
            message: format!("Port {LOCAL_RUNTIME_PORT} already has a listener. Use the saved local target to connect."),
        });
    }

    let port = LOCAL_RUNTIME_PORT.to_string();
    let mut command = Command::new("cargo");
    command
        .current_dir(repo)
        .args([
            "run",
            "-p",
            "codeorbit-host",
            "--",
            "--token",
            LOCAL_RUNTIME_TOKEN,
            "--port",
            port.as_str(),
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }

    command
        .spawn()
        .map_err(|err| format!("Could not start local Runtime with cargo: {err}"))?;

    Ok(LocalRuntimeStartResult {
        success: true,
        message: format!(
            "Starting CodeOrbit Rust Runtime on http://127.0.0.1:{LOCAL_RUNTIME_PORT}."
        ),
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            list_runtime_targets,
            save_runtime_target,
            delete_runtime_target,
            get_runtime_target_token,
            start_local_runtime
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn targets_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|err| format!("Could not resolve app config directory: {err}"))?;
    fs::create_dir_all(&dir)
        .map_err(|err| format!("Could not create app config directory: {err}"))?;
    Ok(dir.join(TARGETS_FILE))
}

fn read_targets(app: &AppHandle) -> Result<RuntimeTargetsFile, String> {
    let path = targets_path(app)?;
    if !path.exists() {
        return Ok(RuntimeTargetsFile::default());
    }

    let text = fs::read_to_string(&path)
        .map_err(|err| format!("Could not read runtime targets: {err}"))?;
    serde_json::from_str(&text).map_err(|err| format!("Could not parse runtime targets: {err}"))
}

fn write_targets(app: &AppHandle, file: &RuntimeTargetsFile) -> Result<(), String> {
    let path = targets_path(app)?;
    let text = serde_json::to_string_pretty(file)
        .map_err(|err| format!("Could not serialize runtime targets: {err}"))?;
    fs::write(path, text).map_err(|err| format!("Could not write runtime targets: {err}"))
}

fn token_entry(id: &str) -> Result<Entry, String> {
    Entry::new(KEYRING_SERVICE, id)
        .map_err(|err| format!("Could not open OS credential storage: {err}"))
}

fn normalize_base_url(value: &str) -> Result<String, String> {
    let trimmed = value.trim().trim_end_matches('/').to_string();
    if trimmed.is_empty() {
        return Err("Runtime URL is required.".into());
    }

    if !(trimmed.starts_with("http://") || trimmed.starts_with("https://")) {
        return Err("Runtime URL must start with http:// or https://.".into());
    }

    Ok(trimmed)
}

fn now_stamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".into())
}

fn new_target_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("target-{nanos}-{}", std::process::id())
}
