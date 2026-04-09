use crate::dispatcher::AppController;
use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

const ICON_BYTES: &[u8] = include_bytes!("../icons/32x32.png");

fn load_tray_icon() -> Icon {
    let decoder = png::Decoder::new(std::io::Cursor::new(ICON_BYTES));
    let mut reader = decoder.read_info().expect("embedded PNG header must be valid");
    let mut buf = vec![0u8; reader.output_buffer_size().expect("PNG output buffer size must be known")];
    let info = reader.next_frame(&mut buf).expect("embedded PNG frame must be valid");
    buf.truncate(info.buffer_size());

    let (w, h) = (info.width, info.height);
    if info.color_type != png::ColorType::Rgba {
        panic!(
            "tray icon must be RGBA, got {:?}; re-export icons/32x32.png as RGBA",
            info.color_type
        );
    }
    Icon::from_rgba(buf, w, h).expect("icon RGBA data must be valid")
}

/// Build and return the tray icon. The caller MUST hold the returned
/// `TrayIcon` for the lifetime of the application — dropping it removes
/// the icon from the menu bar.
pub fn setup_tray(controller: AppController) -> TrayIcon {
    let menu = build_tray_menu(&controller);
    let icon = load_tray_icon();

    let tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_icon(icon)
        .with_tooltip("Surrogate Proxy")
        .build()
        .expect("failed to build tray icon");

    let ctrl = controller.clone();
    std::thread::spawn(move || {
        loop {
            if let Ok(event) = MenuEvent::receiver().recv() {
                handle_menu_event(&event, &ctrl);
            }
        }
    });

    tray
}

fn build_tray_menu(controller: &AppController) -> Menu {
    let menu = Menu::new();

    let running = controller.is_running();
    let status_text = if running {
        let status = controller.status();
        format!(
            "Surrogate: {}",
            status.listen_addr.as_deref().unwrap_or("Running")
        )
    } else {
        "Surrogate: Stopped".to_string()
    };

    let status_item = MenuItem::with_id("status", &status_text, false, None);
    let _ = menu.append(&status_item);

    let default_ob = controller.default_outbound_id();
    let default_item = MenuItem::with_id(
        "default_outbound",
        &format!("Default: {default_ob}"),
        false,
        None,
    );
    let _ = menu.append(&default_item);

    let (_, _, error_count) = controller.event_counts();
    if error_count > 0 {
        let err_item = MenuItem::with_id(
            "error_count",
            &format!("Errors: {error_count}"),
            false,
            None,
        );
        let _ = menu.append(&err_item);
    }

    let _ = menu.append(&PredefinedMenuItem::separator());

    let toggle_text = if running { "Stop Proxy" } else { "Start Proxy" };
    let toggle = MenuItem::with_id("toggle_proxy", toggle_text, true, None);
    let sys_proxy = MenuItem::with_id("toggle_sys_proxy", "Toggle System Proxy", true, None);
    let _ = menu.append(&toggle);
    let _ = menu.append(&sys_proxy);

    let _ = menu.append(&PredefinedMenuItem::separator());

    let open_window = MenuItem::with_id("open_window", "Open Main Window", true, None);
    let copy_export = MenuItem::with_id("copy_export", "Copy Export Command", true, None);
    let _ = menu.append(&open_window);
    let _ = menu.append(&copy_export);

    let _ = menu.append(&PredefinedMenuItem::separator());

    let mode_simple = MenuItem::with_id("mode_simple", "Simple", true, None);
    let mode_advanced = MenuItem::with_id("mode_advanced", "Advanced", true, None);
    let mode_expert = MenuItem::with_id("mode_expert", "Expert", true, None);
    let mode_submenu = Submenu::with_id("mode", "Mode", true);
    let _ = mode_submenu.append(&mode_simple);
    let _ = mode_submenu.append(&mode_advanced);
    let _ = mode_submenu.append(&mode_expert);
    let _ = menu.append(&mode_submenu);

    let _ = menu.append(&PredefinedMenuItem::separator());

    let quick_test = MenuItem::with_id("quick_test_panel", "Quick: Test Panel", true, None);
    let quick_observe = MenuItem::with_id("quick_observe", "Quick: Observe", true, None);
    let _ = menu.append(&quick_test);
    let _ = menu.append(&quick_observe);

    let _ = menu.append(&PredefinedMenuItem::separator());

    let quit = MenuItem::with_id("quit", "Quit Surrogate", true, None);
    let _ = menu.append(&quit);

    menu
}

fn handle_menu_event(event: &MenuEvent, controller: &AppController) {
    match event.id().as_ref() {
        "quit" => {
            controller.cleanup_and_exit();
            std::process::exit(0);
        }
        "toggle_proxy" => {
            if controller.is_running() {
                if let Err(e) = controller.stop_proxy() {
                    eprintln!("[surrogate] stop failed: {e}");
                }
            } else if let Err(e) = controller.start_proxy() {
                eprintln!("[surrogate] start failed: {e}");
            }
        }
        "toggle_sys_proxy" => {
            let currently_enabled = controller.is_system_proxy_enabled();
            if let Err(e) = controller.toggle_system_proxy(!currently_enabled) {
                eprintln!("[surrogate] system proxy toggle failed: {e}");
            }
        }
        "copy_export" => {
            if let Some(cmd) = controller.export_command() {
                write_to_clipboard(&cmd);
            }
        }
        "mode_simple" => {
            controller.set_ui_mode(crate::dispatcher::UiMode::Simple);
        }
        "mode_advanced" => {
            controller.set_ui_mode(crate::dispatcher::UiMode::Advanced);
        }
        "mode_expert" => {
            controller.set_ui_mode(crate::dispatcher::UiMode::Expert);
        }
        "open_window" => {
            activate_app_window();
        }
        "quick_test_panel" => {
            eprintln!("[surrogate] tray: Quick Test Panel selected (placeholder — cannot switch window tab from tray)");
        }
        "quick_observe" => {
            eprintln!("[surrogate] tray: Quick Observe selected (placeholder — cannot switch window tab from tray)");
        }
        unknown => {
            eprintln!("[surrogate] unhandled menu event: {unknown}");
        }
    }
}

/// Bring the Surrogate main window to front by activating the process.
/// Uses osascript because the tray event thread cannot safely call AppKit
/// APIs that belong to the main thread's run loop.
fn activate_app_window() {
    if let Err(e) = std::process::Command::new("osascript")
        .args([
            "-e",
            "tell application \"System Events\" to set frontmost of process \"Surrogate\" to true",
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
    {
        eprintln!("[surrogate] failed to activate window: {e}");
    }
}

fn write_to_clipboard(text: &str) {
    use std::io::Write;
    if let Ok(mut child) = std::process::Command::new("pbcopy")
        .stdin(std::process::Stdio::piped())
        .spawn()
    {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(text.as_bytes());
        }
        let _ = child.wait();
    }
}
