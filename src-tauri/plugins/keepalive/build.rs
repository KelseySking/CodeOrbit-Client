const COMMANDS: &[&str] = &[
    "keep_alive_start",
    "keep_alive_update",
    "keep_alive_stop",
    "pending_alert_notify",
    "pending_alert_clear",
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS)
        .android_path("android")
        .build();
}
