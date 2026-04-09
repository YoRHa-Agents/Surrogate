use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};
use surrogate_app::ProxyManager;
use surrogate_contract::config::{load_and_validate, normalize, serialize_normalized};
use tokio::signal::unix::{signal, SignalKind};

const DEFAULT_CONFIG: &str = r#"listen = "127.0.0.1:41080"
default_outbound = "direct"

[[outbounds]]
id = "direct"
type = "direct"

[[outbounds]]
id = "reject"
type = "reject"
"#;

// ---------------------------------------------------------------------------
// .app bundle mismatch detection
// ---------------------------------------------------------------------------

fn is_inside_app_bundle() -> bool {
    std::env::current_exe()
        .map(|exe| exe.to_string_lossy().contains(".app/Contents/MacOS/"))
        .unwrap_or(false)
}

fn block_if_running_inside_app_bundle() {
    if !is_inside_app_bundle() {
        return;
    }

    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("osascript")
            .args([
                "-e",
                concat!(
                    "display dialog ",
                    "\"This .app contains the wrong binary (surrogate-app).\\n\\n",
                    "The macOS GUI requires surrogate-macos.\\n",
                    "Rebuild with:\\n",
                    "  cargo build -p surrogate-macos\\n",
                    "  ./scripts/package-macos.sh\" ",
                    "with title \"Surrogate — Wrong Binary\" ",
                    "buttons {\"OK\"} default button \"OK\" with icon stop",
                ),
            ])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    }

    eprintln!("[surrogate] FATAL: surrogate-app running inside .app bundle.");
    eprintln!("[surrogate] The macOS GUI requires the surrogate-macos binary.");
    eprintln!("[surrogate] Rebuild: cargo build -p surrogate-macos && ./scripts/package-macos.sh");
    std::process::exit(1);
}

// ---------------------------------------------------------------------------
// Entry point and commands
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> Result<()> {
    block_if_running_inside_app_bundle();

    let mut args = std::env::args().skip(1);
    let command = args.next().unwrap_or_default();

    match command.as_str() {
        "validate-config" => {
            let path = require_config_path(args.next())?;
            load_and_validate(&path)
                .with_context(|| format!("failed to validate config at `{}`", path.display()))?;
            println!("config valid: {}", path.display());
            Ok(())
        }
        "dump-normalized" => {
            let path = require_config_path(args.next())?;
            let document = load_and_validate(&path)
                .with_context(|| format!("failed to validate config at `{}`", path.display()))?;
            let normalized = normalize(&document);
            let rendered = serialize_normalized(&normalized)?;
            println!("{rendered}");
            Ok(())
        }
        "serve" => {
            let path = parse_serve_cli_args(args)?;
            run_serve(&path).await
        }
        "--help" | "-h" | "help" => {
            print_usage();
            Ok(())
        }
        "--version" | "-V" => {
            println!("surrogate-app {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        "" => run_default().await,
        _ => {
            let as_path = PathBuf::from(&command);
            if as_path.exists() && as_path.extension().is_some_and(|e| e == "toml") {
                run_serve(&as_path).await
            } else {
                print_usage();
                bail!("unknown command `{command}`. See usage above.");
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Core proxy lifecycle (now using ProxyManager)
// ---------------------------------------------------------------------------

async fn run_default() -> Result<()> {
    let config_path = discover_or_create_config()?;
    eprintln!("[surrogate] using config: {}", config_path.display());
    run_serve(&config_path).await
}

async fn run_serve(path: &Path) -> Result<()> {
    let manager = ProxyManager::new(path.to_path_buf());

    let mut event_rx = manager.subscribe_events();
    tokio::spawn(async move {
        while let Ok(event) = event_rx.recv().await {
            if let Ok(line) = serde_json::to_string(&event) {
                println!("{line}");
            }
        }
    });

    let listen = manager
        .start()
        .await
        .with_context(|| {
            "failed to start listener -- if port is already in use, \
             another Surrogate instance may be running. \
             Check with: lsof -i :41080"
        })?;

    eprintln!("[surrogate] proxy listening on {listen}  (press Ctrl+C to stop)");
    write_startup_log(path, &listen.to_string());

    let mut sigterm = signal(SignalKind::terminate())
        .context("failed to register SIGTERM handler")?;
    tokio::select! {
        result = tokio::signal::ctrl_c() => {
            result.context("failed while waiting for ctrl-c")?;
        }
        _ = sigterm.recv() => {}
    }
    eprintln!("\n[surrogate] shutting down...");
    manager
        .stop()
        .await
        .context("failed while shutting down proxy")?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Startup logging
// ---------------------------------------------------------------------------

fn write_startup_log(config_path: &Path, listen_addr: &str) {
    let log_dir = if cfg!(target_os = "macos") {
        home_dir().map(|h| h.join("Library").join("Logs").join("Surrogate"))
    } else {
        home_dir().map(|h| h.join(".local").join("share").join("surrogate").join("logs"))
    };
    if let Some(dir) = log_dir {
        let _ = std::fs::create_dir_all(&dir);
        let log_path = dir.join("surrogate.log");
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let entry = format!(
            "[{}] started | config={} | listen={}\n",
            timestamp,
            config_path.display(),
            listen_addr
        );
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .and_then(|mut f| std::io::Write::write_all(&mut f, entry.as_bytes()));
    }
}

// ---------------------------------------------------------------------------
// Config discovery
// ---------------------------------------------------------------------------

fn discover_or_create_config() -> Result<PathBuf> {
    for candidate in config_search_paths() {
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    let target = default_config_dir()?.join("config.toml");
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create config directory `{}`", parent.display()))?;
    }
    std::fs::write(&target, DEFAULT_CONFIG)
        .with_context(|| format!("failed to write default config to `{}`", target.display()))?;
    eprintln!("[surrogate] created default config at {}", target.display());
    Ok(target)
}

fn config_search_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Some(dir) = exe_sibling_dir() {
        paths.push(dir.join("config.toml"));
    }

    if let Ok(dir) = default_config_dir() {
        paths.push(dir.join("config.toml"));
    }

    if let Some(home) = home_dir() {
        paths.push(home.join(".config").join("surrogate").join("config.toml"));
    }

    paths
}

fn default_config_dir() -> Result<PathBuf> {
    if cfg!(target_os = "macos") {
        let home = home_dir().context("could not determine home directory")?;
        Ok(home
            .join("Library")
            .join("Application Support")
            .join("Surrogate"))
    } else {
        let home = home_dir().context("could not determine home directory")?;
        Ok(home.join(".config").join("surrogate"))
    }
}

fn exe_sibling_dir() -> Option<PathBuf> {
    std::env::current_exe().ok()?.parent().map(PathBuf::from)
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

fn require_config_path(path: Option<String>) -> Result<PathBuf> {
    match path {
        Some(path) => Ok(PathBuf::from(path)),
        None => {
            print_usage();
            bail!("missing config path argument")
        }
    }
}

fn parse_serve_cli_args(args: impl Iterator<Item = String>) -> Result<PathBuf> {
    let mut config_path: Option<PathBuf> = None;
    for arg in args {
        if config_path.is_some() {
            print_usage();
            bail!("unexpected extra argument `{arg}`");
        }
        config_path = Some(PathBuf::from(arg));
    }
    match config_path {
        Some(path) => Ok(path),
        None => {
            print_usage();
            bail!("missing config path argument")
        }
    }
}

fn print_usage() {
    let version = env!("CARGO_PKG_VERSION");
    eprintln!("Surrogate v{version} — cross-platform proxy kernel");
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("  surrogate-app                          Start proxy with auto-discovered config");
    eprintln!("  surrogate-app serve <config-path>            Start proxy with explicit config");
    eprintln!("  surrogate-app validate-config <path>   Validate a config file");
    eprintln!("  surrogate-app dump-normalized <path>   Print normalized config as JSON");
    eprintln!("  surrogate-app <config-path.toml>       Start proxy with given TOML file");
    eprintln!("  surrogate-app --help                   Show this help");
    eprintln!("  surrogate-app --version                Show version");
    eprintln!();
    eprintln!("CONFIG SEARCH ORDER:");
    eprintln!("  1. <binary-dir>/config.toml");
    if cfg!(target_os = "macos") {
        eprintln!("  2. ~/Library/Application Support/Surrogate/config.toml");
    } else {
        eprintln!("  2. ~/.config/surrogate/config.toml");
    }
    eprintln!("  If no config found, a default one is created automatically.");
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid_toml() {
        let doc: surrogate_contract::config::ConfigDocument =
            toml::from_str(DEFAULT_CONFIG).expect("DEFAULT_CONFIG must be valid TOML");
        surrogate_contract::config::validate(&doc).expect("DEFAULT_CONFIG must pass validation");
    }

    #[test]
    fn config_search_paths_returns_nonempty() {
        let paths = config_search_paths();
        assert!(!paths.is_empty());
    }

    #[test]
    fn discover_creates_config_when_missing() {
        let dir = std::env::temp_dir().join(format!(
            "surrogate-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let config_path = dir.join("config.toml");
        std::fs::write(&config_path, DEFAULT_CONFIG).unwrap();
        assert!(config_path.exists());
        let content = std::fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("127.0.0.1:41080"));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn app_bundle_detection_works() {
        assert!(
            !is_inside_app_bundle(),
            "test binary should not appear inside .app"
        );
    }

    #[test]
    fn serve_arg_parsing() {
        let path =
            parse_serve_cli_args(vec!["b.toml".to_string()].into_iter()).expect("path only");
        assert_eq!(path, PathBuf::from("b.toml"));

        assert!(parse_serve_cli_args(std::iter::empty()).is_err());
        assert!(parse_serve_cli_args(
            vec!["x.toml".to_string(), "y.toml".to_string()].into_iter()
        )
        .is_err());
    }
}
