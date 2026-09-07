use std::path::{Component, Path, PathBuf};

use tauri::http::{Request, Response, StatusCode};
use tauri::{AppHandle, Manager, Runtime, UriSchemeContext};

const HOST_SDK_JS: &str =
    include_str!("../../../../../static/plugin-sdk/plugin-host-sdk.js");

pub fn handle<R: Runtime>(
    ctx: UriSchemeContext<'_, R>,
    request: Request<Vec<u8>>,
) -> Response<Vec<u8>> {
    let app = ctx.app_handle();
    let uri = request.uri();
    let host = uri.host().unwrap_or("");
    let path = uri.path().trim_start_matches('/');

    // Special segment: `plugin://__host__/plugin-sdk/plugin-host-sdk.js`
    // serves the embedded host SDK without touching the filesystem.
    if host == "__host__" && path == "plugin-sdk/plugin-host-sdk.js" {
        return Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/javascript; charset=utf-8")
            .body(HOST_SDK_JS.as_bytes().to_vec())
            .unwrap();
    }

    // Validate the plugin id: must be non-empty and contain no path
    // separators or traversal segments.
    let plugin_id = match host {
        "" => return not_found("missing plugin id"),
        id if id.contains('/') || id.contains("..") || id.contains('\0') => {
            return forbidden("invalid plugin id")
        }
        id => id,
    };

    // Cross-plugin isolation: when a sub-resource request carries a
    // `Referer` whose plugin id differs from the requested plugin id,
    // reject with 403. Entry HTML loads have no Referer and are allowed.
    if let Some(referer) = request.headers().get("referer").and_then(|v| v.to_str().ok()) {
        if let Some(referer_plugin_id) = parse_plugin_referer(referer) {
            if referer_plugin_id != plugin_id {
                return forbidden("cross-plugin access denied");
            }
        }
    }

    // Path safety: reject absolute paths, traversal, and NUL bytes up front.
    if path.is_empty() || path.starts_with('/') || path.contains("..") || path.contains('\0') {
        return forbidden("invalid path");
    }

    // Component-level check: only `Normal` components are allowed. This
    // rejects `RootDir`, `ParentDir`, `Prefix` (Windows drive), and
    // `CurDir` (`.`).
    let rel = Path::new(path);
    for comp in rel.components() {
        match comp {
            Component::Normal(_) => {}
            _ => return forbidden("invalid path component"),
        }
    }

    // Locate the plugin's resource root directory.
    let root = match resolve_plugin_root(app, plugin_id) {
        Some(p) => p,
        None => return not_found("plugin root not found"),
    };

    let file_path = root.join(rel);

    // Canonicalize both the file and the root to defend against symlinks or
    // other filesystem tricks that could escape the plugin sandbox.
    let canonical = match file_path.canonicalize() {
        Ok(p) => p,
        Err(_) => return not_found("file not found"),
    };
    let canonical_root = match root.canonicalize() {
        Ok(p) => p,
        Err(_) => return not_found("plugin root invalid"),
    };
    if !canonical.starts_with(&canonical_root) {
        return forbidden("path escapes plugin root");
    }

    match std::fs::read(&canonical) {
        Ok(bytes) => Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", content_type_for(&canonical))
            .body(bytes)
            .unwrap(),
        Err(_) => not_found("file read error"),
    }
}

fn parse_plugin_referer(referer: &str) -> Option<&str> {
    let stripped = referer.strip_prefix("plugin://")?;
    let plugin_id = stripped.split('/').next()?;
    if plugin_id.is_empty() || plugin_id.contains("..") || plugin_id.contains('\0') {
        return None;
    }
    Some(plugin_id)
}

fn resolve_plugin_root<R: Runtime>(app: &AppHandle<R>, plugin_id: &str) -> Option<PathBuf> {
    let mut candidates = Vec::new();

    if let Ok(app_data) = app.path().app_data_dir() {
        candidates.push(app_data.join("plugins").join(plugin_id));
    }

    if let Ok(resource_dir) = app.path().resource_dir() {
        candidates.push(resource_dir.join("plugins").join(plugin_id));
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            candidates.push(parent.join(plugin_id));
        }
    }

    candidates.into_iter().find(|p| p.exists())
}

fn content_type_for(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("js") => "application/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("svg") => "image/svg+xml",
        _ => "application/octet-stream",
    }
}

fn not_found(msg: &str) -> Response<Vec<u8>> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(msg.as_bytes().to_vec())
        .unwrap()
}

fn forbidden(msg: &str) -> Response<Vec<u8>> {
    Response::builder()
        .status(StatusCode::FORBIDDEN)
        .body(msg.as_bytes().to_vec())
        .unwrap()
}
