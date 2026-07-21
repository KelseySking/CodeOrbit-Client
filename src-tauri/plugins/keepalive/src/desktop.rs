use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime};

use crate::models::*;

pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> crate::Result<Keepalive<R>> {
    Ok(Keepalive(app.clone()))
}

pub struct Keepalive<R: Runtime>(AppHandle<R>);

impl<R: Runtime> Keepalive<R> {
    pub fn keep_alive_start(&self, _payload: KeepAlivePayload) -> crate::Result<()> {
        Ok(())
    }

    pub fn keep_alive_update(&self, _payload: KeepAlivePayload) -> crate::Result<()> {
        Ok(())
    }

    pub fn keep_alive_stop(&self) -> crate::Result<()> {
        Ok(())
    }

    pub fn pending_alert_notify(&self, _payload: PendingAlertPayload) -> crate::Result<()> {
        Ok(())
    }

    pub fn pending_alert_clear(&self) -> crate::Result<()> {
        Ok(())
    }
}
