use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};
use surrogate_contract::config::{load_and_validate, normalize, serialize_normalized};
use surrogate_contract::events::Observability;
use surrogate_kernel::Kernel;

const DEFAULT_CONFIG: &str = r#"listen = "127.0.0.1:41080"
default_outbound = "direct"

[[outbounds]]
id = "direct"
type = "direct"

[[outbounds]]
id = "reject"
type = "reject"
"#;

#[tokio::main]
async fn main() -> Result<()> {
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
            let path = require_config_path(args.next())?;
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

async fn run_default() -> Result<()> {
    let config_path = discover_or_create_config()?;
    eprintln!("[surrogate] using config: {}", config_path.display());
    run_serve(&config_path).await
}

async fn run_serve(path: &Path) -> Result<()> {
    let document = load_and_validate(path)
        .with_context(|| format!("failed to validate config at `{}`", path.display()))?;
    let kernel = Kernel::new(normalize(&document), Observability::stdout())
        .context("failed to build kernel from normalized config")?;
    let running = kernel
        .spawn()
        .await
        .context("failed to start kernel listener")?;

    eprintln!(
        "[surrogate] proxy listening on {}  (press Ctrl+C to stop)",
        running.local_addr()
    );
    tokio::signal::ctrl_c()
        .await
        .context("failed while waiting for ctrl-c")?;
    eprintln!("\n[surrogate] shutting down...");
    running
        .shutdown()
        .await
        .context("failed while shutting down kernel")?;
    Ok(())
}

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

fn print_usage() {
    let version = env!("CARGO_PKG_VERSION");
    eprintln!("Surrogate v{version} — cross-platform proxy kernel");
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("  surrogate-app                          Start proxy with auto-discovered config");
    eprintln!("  surrogate-app serve <config-path>      Start proxy with explicit config");
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
}
