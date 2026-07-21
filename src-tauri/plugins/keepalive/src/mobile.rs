use serde::de::DeserializeOwned;
use tauri::{
    plugin::{PluginApi, PluginHandle},
    AppHandle, Runtime,
};

use crate::models::*;

#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "com.codeorbit.keepalive";

pub fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    api: PluginApi<R, C>,
) -> crate::Result<Keepalive<R>> {
    #[cfg(target_os = "android")]
    {
        let handle = api.register_android_plugin(PLUGIN_IDENTIFIER, "KeepAlivePlugin")?;
        return Ok(Keepalive(handle));
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = api;
        Err(crate::Error::Io(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "keepalive mobile bridge is Android-only",
        )))
    }
}

pub struct Keepalive<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> Keepalive<R> {
    pub fn keep_alive_start(&self, payload: KeepAlivePayload) -> crate::Result<()> {
        self.0
            .run_mobile_plugin("keepAliveStart", payload)
            .map_err(Into::into)
    }

    pub fn keep_alive_update(&self, payload: KeepAlivePayload) -> crate::Result<()> {
        self.0
            .run_mobile_plugin("keepAliveUpdate", payload)
            .map_err(Into::into)
    }

    pub fn keep_alive_stop(&self) -> crate::Result<()> {
        self.0
            .run_mobile_plugin("keepAliveStop", ())
            .map_err(Into::into)
    }

    pub fn pending_alert_notify(&self, payload: PendingAlertPayload) -> crate::Result<()> {
        self.0
            .run_mobile_plugin("pendingAlertNotify", payload)
            .map_err(Into::into)
    }

    pub fn pending_alert_clear(&self) -> crate::Result<()> {
        self.0
            .run_mobile_plugin("pendingAlertClear", ())
            .map_err(Into::into)
    }
}
