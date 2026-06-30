mod app_state;
mod commands;
mod companion;

use tauri::Manager;
use tauri_plugin_deep_link::DeepLinkExt;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_tracing();

    if let Err(e) = core_contracts::assert_contracts_present() {
        tracing::error!("{e}");
    }

    let mut builder = tauri::Builder::default();

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            let _ = app.get_webview_window("main").map(|w| w.set_focus());
        }));
    }

    builder
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_deep_link::init())
        .setup(|app| {
            let state = app_state::AppState::initialize(app.handle())?;
            app.manage(state);

            #[cfg(any(windows, target_os = "linux"))]
            {
                if let Err(e) = app.deep_link().register_all() {
                    tracing::warn!("deep link register_all: {e}");
                }
            }

            tracing::info!("Brian Desktop scaffold ready");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::health_check,
            commands::contracts_info,
            commands::auth_start_apple,
            commands::auth_complete_oauth,
            commands::auth_sign_out,
            commands::auth_session_info,
            commands::list_tasks,
            commands::list_timeline,
            commands::list_widgets,
            commands::create_task,
            commands::sync_drain,
            commands::sync_pull,
            commands::widget_catalog,
            commands::install_widget,
            commands::update_widget_layouts,
            commands::remove_widget,
            commands::seed_desktop_layout_if_empty,
            commands::reset_desktop_layout,
            commands::repair_workbench_layout,
            commands::get_ui_prefs,
            commands::set_ui_prefs,
            commands::get_shell_layout,
            commands::set_shell_layout,
            commands::get_assist_prefs,
            commands::set_assist_prefs,
            companion::get_browser_state,
            companion::set_browser_state,
            companion::list_notes,
            companion::create_note,
            companion::update_note,
            companion::get_ide_state,
            companion::set_ide_state,
            companion::pick_ide_folder,
            companion::list_ide_files,
            companion::read_ide_file,
            companion::write_ide_file,
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
