mod app_state;
mod commands;

use tauri::Manager;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_tracing();

    if let Err(e) = core_contracts::assert_contracts_present() {
        tracing::error!("{e}");
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let state = app_state::AppState::initialize(app.handle())?;
            app.manage(state);
            tracing::info!("Brian Desktop scaffold ready");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::health_check,
            commands::contracts_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Brian Desktop");
}

fn init_tracing() {
    let _ = tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "brian_desktop_lib=info,tauri=warn".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .try_init();
}
