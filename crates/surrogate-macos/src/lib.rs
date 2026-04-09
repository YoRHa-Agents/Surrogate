pub mod dispatcher;
pub mod navigation;
pub mod theme;
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
mod event_format;

#[cfg(target_os = "macos")]
mod tray;
#[cfg(target_os = "macos")]
mod window;
#[cfg(target_os = "macos")]
mod pages;

use std::path::PathBuf;

#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn default_config_path() -> PathBuf {
    if let Some(home) = std::env::var_os("HOME") {
        let path = PathBuf::from(&home)
            .join("Library")
            .join("Application Support")
            .join("Surrogate")
            .join("config.toml");
        if path.exists() {
            return path;
        }
    }

    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        let path = dir.join("config.toml");
        if path.exists() {
            return path;
        }
    }

    if let Some(home) = std::env::var_os("HOME") {
        let macos_path = PathBuf::from(&home)
            .join("Library")
            .join("Application Support")
            .join("Surrogate")
            .join("config.toml");
        if let Some(parent) = macos_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let default_config = "\
listen = \"127.0.0.1:41080\"\n\
default_outbound = \"direct\"\n\
\n\
[[outbounds]]\n\
id = \"direct\"\n\
type = \"direct\"\n\
\n\
[[outbounds]]\n\
id = \"reject\"\n\
type = \"reject\"\n";
        let _ = std::fs::write(&macos_path, default_config);
        return macos_path;
    }

    PathBuf::from("config.toml")
}

#[cfg(target_os = "macos")]
pub fn run() {
    use dispatcher::AppController;

    let config_path = default_config_path();
    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
    let controller = AppController::new(config_path, rt);

    let controller_for_tray = controller.clone();
    let controller_for_window = controller.clone();

    std::thread::spawn(move || {
        controller.auto_start_proxy();
    });

    // _tray_icon must be held for the lifetime of the app — dropping it
    // removes the icon from the menu bar.
    let _tray_icon = tray::setup_tray(controller_for_tray);

    std::thread::spawn(|| {
        window::configure_main_window();
    });

    window::run_app(controller_for_window);
}

#[cfg(not(target_os = "macos"))]
pub fn run() {
    eprintln!("Surrogate macOS GUI is only available on macOS.");
    eprintln!("Use `surrogate-app` CLI instead.");
    std::process::exit(1);
}
