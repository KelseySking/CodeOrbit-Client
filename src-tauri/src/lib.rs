use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{Read, Write},
    net::{TcpStream, ToSocketAddrs},
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::{Mutex, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Manager};

const TARGETS_FILE: &str = "runtime-targets.json";
const KEYRING_SERVICE: &str = "CodeOrbit Client Runtime Target";
const LOCAL_RUNTIME_REPO: &str = r"D:\OtherWork\CodeOrbit-Rust";
const LOCAL_RUNTIME_TOKEN: &str = "dev-token";
const LOCAL_RUNTIME_PORT: u16 = 32145;
static LOCAL_RUNTIME_CHILD: OnceLock<Mutex<Option<Child>>> = OnceLock::new();

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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RuntimeProxyRequest {
    method: String,
    url: String,
    token: Option<String>,
    body: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RuntimeProxyResponse {
    status: u16,
    body: String,
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
async fn runtime_request(request: RuntimeProxyRequest) -> Result<RuntimeProxyResponse, String> {
    tauri::async_runtime::spawn_blocking(move || runtime_request_blocking(request))
        .await
        .map_err(|err| format!("Runtime request task failed: {err}"))?
}

#[tauri::command]
fn start_local_runtime() -> Result<LocalRuntimeStartResult, String> {
    let mut child_slot = local_runtime_child().lock().map_err(|_| {
        "Could not lock local Runtime process state. Restart CodeOrbit Client.".to_string()
    })?;
    if let Some(child) = child_slot.as_mut() {
        match child.try_wait() {
            Ok(Some(_)) => *child_slot = None,
            Ok(None) => {
                return Ok(LocalRuntimeStartResult {
                    success: true,
                    message: "Local Runtime was already started by this client.".into(),
                });
            }
            Err(err) => return Err(format!("Could not inspect local Runtime process: {err}")),
        }
    }

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

    let child = command
        .spawn()
        .map_err(|err| format!("Could not start local Runtime with cargo: {err}"))?;
    *child_slot = Some(child);

    Ok(LocalRuntimeStartResult {
        success: true,
        message: format!(
            "Starting CodeOrbit Rust Runtime on http://127.0.0.1:{LOCAL_RUNTIME_PORT}."
        ),
    })
}

#[tauri::command]
fn stop_local_runtime() -> Result<LocalRuntimeStartResult, String> {
    let mut child_slot = local_runtime_child().lock().map_err(|_| {
        "Could not lock local Runtime process state. Restart CodeOrbit Client.".to_string()
    })?;
    let Some(child) = child_slot.as_mut() else {
        return Ok(LocalRuntimeStartResult {
            success: false,
            message: "No local Runtime process started by this client.".into(),
        });
    };

    if child
        .try_wait()
        .map_err(|err| format!("Could not inspect local Runtime process: {err}"))?
        .is_some()
    {
        *child_slot = None;
        return Ok(LocalRuntimeStartResult {
            success: true,
            message: "Local Runtime process had already exited.".into(),
        });
    }

    #[cfg(windows)]
    {
        let pid = child.id().to_string();
        let status = Command::new("taskkill")
            .args(["/PID", pid.as_str(), "/T", "/F"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|err| format!("Could not stop local Runtime process tree: {err}"))?;
        if !status.success() {
            return Err(format!(
                "taskkill failed while stopping local Runtime process {pid}."
            ));
        }
    }

    #[cfg(not(windows))]
    {
        child
            .kill()
            .map_err(|err| format!("Could not stop local Runtime process: {err}"))?;
    }

    let _ = child.wait();
    *child_slot = None;
    Ok(LocalRuntimeStartResult {
        success: true,
        message: "Local Runtime stopped.".into(),
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
            runtime_request,
            start_local_runtime,
            stop_local_runtime
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn local_runtime_child() -> &'static Mutex<Option<Child>> {
    LOCAL_RUNTIME_CHILD.get_or_init(|| Mutex::new(None))
}

struct ParsedHttpUrl {
    host: String,
    port: u16,
    path: String,
}

fn runtime_request_blocking(request: RuntimeProxyRequest) -> Result<RuntimeProxyResponse, String> {
    let parsed = parse_http_url(&request.url)?;
    let method = match request.method.trim().to_ascii_uppercase().as_str() {
        "GET" => "GET",
        "POST" => "POST",
        _ => return Err("Runtime request method is invalid.".into()),
    };
    let body = request
        .body
        .map(|body| body.to_string())
        .unwrap_or_default();
    let host_header = if parsed.port == 80 {
        parsed.host.clone()
    } else {
        format!("{}:{}", parsed.host, parsed.port)
    };

    let mut raw = format!(
        "{method} {} HTTP/1.1\r\nHost: {host_header}\r\nAccept: application/json\r\nConnection: close\r\n",
        parsed.path
    );
    if let Some(token) = request
        .token
        .as_deref()
        .map(str::trim)
        .filter(|token| !token.is_empty())
    {
        raw.push_str(&format!("Authorization: Bearer {token}\r\n"));
    }
    if !body.is_empty() {
        raw.push_str("Content-Type: application/json\r\n");
        raw.push_str(&format!("Content-Length: {}\r\n", body.len()));
    }
    raw.push_str("\r\n");
    raw.push_str(&body);

    let mut addresses = (parsed.host.as_str(), parsed.port)
        .to_socket_addrs()
        .map_err(|err| format!("Could not resolve Runtime host: {err}"))?;
    let address = addresses
        .next()
        .ok_or_else(|| "Runtime host did not resolve to an address.".to_string())?;
    let mut stream =
        TcpStream::connect(address).map_err(|err| format!("Runtime request failed: {err}"))?;
    stream
        .write_all(raw.as_bytes())
        .map_err(|err| format!("Could not write Runtime request: {err}"))?;

    let mut response = Vec::new();
    stream
        .read_to_end(&mut response)
        .map_err(|err| format!("Could not read Runtime response: {err}"))?;

    parse_http_response(&response)
}

fn parse_http_url(url: &str) -> Result<ParsedHttpUrl, String> {
    if url.starts_with("https://") {
        return Err(
            "CodeOrbit Client local proxy currently supports http:// Runtime URLs only.".into(),
        );
    }
    let rest = url
        .strip_prefix("http://")
        .ok_or_else(|| "Runtime URL must start with http://.".to_string())?;
    let (host_port, path) = rest
        .split_once('/')
        .map(|(host, path)| (host, format!("/{path}")))
        .unwrap_or((rest, "/".to_string()));
    let (host, port) = if let Some((host, port)) = host_port.rsplit_once(':') {
        (
            host.to_string(),
            port.parse::<u16>()
                .map_err(|_| "Runtime URL port is invalid.".to_string())?,
        )
    } else {
        (host_port.to_string(), 80)
    };

    if host.trim().is_empty() {
        return Err("Runtime URL host is required.".into());
    }

    Ok(ParsedHttpUrl { host, port, path })
}

fn parse_http_response(response: &[u8]) -> Result<RuntimeProxyResponse, String> {
    let header_end = response
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or_else(|| "Runtime response was malformed.".to_string())?;
    let headers = String::from_utf8_lossy(&response[..header_end]);
    let status = headers
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|code| code.parse::<u16>().ok())
        .ok_or_else(|| "Runtime response status was malformed.".to_string())?;
    let mut body = response[header_end + 4..].to_vec();
    if headers
        .lines()
        .any(|line| line.to_ascii_lowercase().trim() == "transfer-encoding: chunked")
    {
        body = decode_chunked_body(&body)?;
    }

    Ok(RuntimeProxyResponse {
        status,
        body: String::from_utf8_lossy(&body).into_owned(),
    })
}

fn decode_chunked_body(body: &[u8]) -> Result<Vec<u8>, String> {
    let mut decoded = Vec::new();
    let mut index = 0;
    loop {
        let line_end = body[index..]
            .windows(2)
            .position(|window| window == b"\r\n")
            .ok_or_else(|| "Runtime chunked response was malformed.".to_string())?
            + index;
        let size_text = String::from_utf8_lossy(&body[index..line_end]);
        let size = usize::from_str_radix(size_text.split(';').next().unwrap_or("").trim(), 16)
            .map_err(|_| "Runtime chunk size was malformed.".to_string())?;
        index = line_end + 2;
        if size == 0 {
            return Ok(decoded);
        }
        let chunk_end = index + size;
        if chunk_end + 2 > body.len() {
            return Err("Runtime chunked response ended early.".into());
        }
        decoded.extend_from_slice(&body[index..chunk_end]);
        index = chunk_end + 2;
    }
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
