use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

pub use models::*;

#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;

mod commands;
mod error;
mod models;

pub use error::{Error, Result};

#[cfg(desktop)]
use desktop::Keepalive;
#[cfg(mobile)]
use mobile::Keepalive;

pub trait KeepaliveExt<R: Runtime> {
    fn keepalive(&self) -> &Keepalive<R>;
}

impl<R: Runtime, T: Manager<R>> crate::KeepaliveExt<R> for T {
    fn keepalive(&self) -> &Keepalive<R> {
        self.state::<Keepalive<R>>().inner()
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("keepalive")
        .invoke_handler(tauri::generate_handler![
            commands::keep_alive_start,
            commands::keep_alive_update,
            commands::keep_alive_stop,
            commands::pending_alert_notify,
            commands::pending_alert_clear,
        ])
        .setup(|app, api| {
            #[cfg(mobile)]
            let keepalive = mobile::init(app, api)?;
            #[cfg(desktop)]
            let keepalive = desktop::init(app, api)?;
            app.manage(keepalive);
            Ok(())
        })
        .build()
}
