use anyhow::{Context, Result, bail};
use std::path::PathBuf;
use surrogate_contract::config::{load_and_validate, normalize, serialize_normalized};
use surrogate_contract::events::Observability;
use surrogate_kernel::Kernel;

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
            let document = load_and_validate(&path)
                .with_context(|| format!("failed to validate config at `{}`", path.display()))?;
            let kernel = Kernel::new(normalize(&document), Observability::stdout())
                .context("failed to build kernel from normalized config")?;
            let running = kernel
                .spawn()
                .await
                .context("failed to start kernel listener")?;

            println!("proxy kernel listening on {}", running.local_addr());
            tokio::signal::ctrl_c()
                .await
                .context("failed while waiting for ctrl-c")?;
            running
                .shutdown()
                .await
                .context("failed while shutting down kernel")?;
            Ok(())
        }
        _ => {
            print_usage();
            bail!("unsupported command `{command}`");
        }
    }
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
    eprintln!("usage: surrogate-app <validate-config|dump-normalized|serve> <config-path>");
}
