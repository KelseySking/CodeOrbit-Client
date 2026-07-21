use tauri::{command, AppHandle, Runtime};

use crate::models::*;
use crate::KeepaliveExt;
use crate::Result;

#[command]
pub(crate) async fn keep_alive_start<R: Runtime>(
    app: AppHandle<R>,
    payload: KeepAlivePayload,
) -> Result<()> {
    app.keepalive().keep_alive_start(payload)
}

#[command]
pub(crate) async fn keep_alive_update<R: Runtime>(
    app: AppHandle<R>,
    payload: KeepAlivePayload,
) -> Result<()> {
    app.keepalive().keep_alive_update(payload)
}

#[command]
pub(crate) async fn keep_alive_stop<R: Runtime>(app: AppHandle<R>) -> Result<()> {
    app.keepalive().keep_alive_stop()
}

#[command]
pub(crate) async fn pending_alert_notify<R: Runtime>(
    app: AppHandle<R>,
    payload: PendingAlertPayload,
) -> Result<()> {
    app.keepalive().pending_alert_notify(payload)
}

#[command]
pub(crate) async fn pending_alert_clear<R: Runtime>(app: AppHandle<R>) -> Result<()> {
    app.keepalive().pending_alert_clear()
}
