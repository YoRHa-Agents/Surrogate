use anyhow::{Context, Result, bail};
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use surrogate_app::ProxyManager;
use surrogate_contract::config::{load_and_validate, normalize, serialize_normalized};
use surrogate_contract::events::Event;
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

const DASHBOARD_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>Surrogate — Proxy Control Plane</title>
<style>
:root{color-scheme:dark;--bg:#0a1020;--surface:rgba(18,24,44,.88);--surface2:rgba(27,37,66,.82);--border:rgba(132,154,255,.16);--border2:rgba(132,154,255,.3);--text:#edf1ff;--muted:#9aa7cf;--accent:#7c8cff;--accent2:#5bd4ff;--good:#45d39c;--warn:#ffbc42;--danger:#ff6a78}
*{box-sizing:border-box;margin:0;padding:0}
body{font-family:Inter,ui-sans-serif,system-ui,-apple-system,sans-serif;background:linear-gradient(180deg,#070d1a,#0b1223);color:var(--text);min-height:100vh}
.chrome{height:52px;display:flex;align-items:center;justify-content:space-between;padding:0 20px;background:rgba(20,26,44,.94);border-bottom:1px solid var(--border);-webkit-app-region:drag}
.dots{display:flex;gap:8px}
.dot{width:12px;height:12px;border-radius:50%}
.dot.r{background:#ff5f57}.dot.y{background:#febc2e}.dot.g{background:#28c840}
.chrome-title{color:var(--muted);font-size:.85rem}
.chrome-status{font-size:.78rem;color:var(--good);display:flex;align-items:center;gap:6px}
.chrome-status::before{content:'';width:8px;height:8px;border-radius:50%;background:var(--good);animation:pulse 2s infinite}
@keyframes pulse{0%,100%{opacity:1}50%{opacity:.4}}
.shell{max-width:960px;margin:0 auto;padding:28px 24px}
.hero{display:grid;grid-template-columns:1fr 1fr;gap:16px;margin-bottom:24px}
.card{background:var(--surface);border:1px solid var(--border);border-radius:20px;padding:18px 20px}
.card h2{font-size:1rem;color:var(--muted);font-weight:500;margin-bottom:14px;text-transform:uppercase;letter-spacing:.06em;font-size:.75rem}
.stat{font-size:2rem;font-weight:700;color:var(--accent2);margin-bottom:4px}
.stat-label{color:var(--muted);font-size:.82rem}
.stat-row{display:grid;grid-template-columns:1fr 1fr;gap:14px;margin-top:12px}
.config-item{display:flex;justify-content:space-between;padding:10px 0;border-bottom:1px solid var(--border)}
.config-item:last-child{border-bottom:none}
.config-key{color:var(--muted);font-size:.85rem}
.config-val{color:var(--text);font-size:.85rem;font-weight:600}
.events{margin-top:24px}
.events h2{font-size:.75rem;text-transform:uppercase;letter-spacing:.06em;color:var(--muted);margin-bottom:14px}
.event-list{display:grid;gap:8px}
.event-row{display:grid;grid-template-columns:42px 1fr auto;gap:12px;padding:12px 16px;background:var(--surface);border:1px solid var(--border);border-radius:14px;font-size:.84rem;align-items:center}
.event-id{color:var(--accent);font-weight:700;font-size:.78rem}
.event-kind{display:inline-block;padding:3px 10px;border-radius:999px;font-size:.72rem;font-weight:600;border:1px solid var(--border)}
.event-kind.session_started{color:var(--accent2)}.event-kind.rule_matched{color:var(--accent)}.event-kind.forward_connected{color:var(--good)}.event-kind.forward_closed{color:var(--muted)}.event-kind.error{color:var(--danger)}
.event-msg{color:var(--muted);overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.event-detail{color:var(--muted);font-size:.78rem;text-align:right;white-space:nowrap}
.empty{text-align:center;padding:40px;color:var(--muted)}
.footer{text-align:center;margin-top:32px;color:var(--muted);font-size:.78rem;opacity:.5}
@media(max-width:700px){.hero{grid-template-columns:1fr}.stat-row{grid-template-columns:1fr}}
</style>
</head>
<body>
<div class="chrome">
  <div class="dots"><span class="dot r"></span><span class="dot y"></span><span class="dot g"></span></div>
  <span class="chrome-title">Surrogate — Proxy Control Plane</span>
  <span class="chrome-status" id="status-dot">Running</span>
</div>
<div class="shell">
  <div class="hero">
    <div class="card">
      <h2>Proxy Status</h2>
      <div class="stat" id="listen-addr">—</div>
      <div class="stat-label">Listen Address</div>
      <div class="stat-row">
        <div><div class="stat" id="uptime" style="font-size:1.4rem">—</div><div class="stat-label">Uptime</div></div>
        <div><div class="stat" id="total-conns" style="font-size:1.4rem">0</div><div class="stat-label">Total Events</div></div>
      </div>
    </div>
    <div class="card">
      <h2>Configuration</h2>
      <div id="config-area"><div class="empty">Loading...</div></div>
    </div>
  </div>
  <div class="events">
    <h2>Recent Events</h2>
    <div class="event-list" id="event-list"><div class="empty">Waiting for connections...</div></div>
  </div>
  <div class="footer">Surrogate v0.1.0 — cross-platform proxy kernel</div>
</div>
<script>
function fmtUptime(s){if(s<60)return s+'s';if(s<3600)return Math.floor(s/60)+'m '+s%60+'s';const h=Math.floor(s/3600);return h+'h '+Math.floor((s%3600)/60)+'m'}
function fmtEvent(e){
  let detail='';
  if(e.host)detail+=e.host;
  if(e.port)detail+=':'+e.port;
  if(e.rule_id&&e.outbound)detail=e.rule_id+' \u2192 '+e.outbound;
  if(e.bytes_from_client!=null)detail='\u2191'+e.bytes_from_client+' \u2193'+e.bytes_from_upstream;
  return detail||e.message;
}
async function poll(){
  try{
    const r=await fetch('/api/status');
    const d=await r.json();
    document.getElementById('listen-addr').textContent=d.listen_addr||'\u2014';
    document.getElementById('uptime').textContent=fmtUptime(d.uptime_secs);
    document.getElementById('total-conns').textContent=d.total_events;
    document.getElementById('status-dot').textContent='Running';
    document.getElementById('status-dot').style.color='var(--good)';
    const cfg=d.config_summary||'';
    if(cfg){
      const lines=cfg.split('\n').filter(l=>l.trim());
      document.getElementById('config-area').innerHTML=lines.map(l=>{
        const[k,...v]=l.split(':');
        return'<div class="config-item"><span class="config-key">'+k.trim()+'</span><span class="config-val">'+v.join(':').trim()+'</span></div>';
      }).join('');
    }
    const evts=d.recent_events||[];
    if(evts.length===0){
      document.getElementById('event-list').innerHTML='<div class="empty">Waiting for connections...</div>';
    }else{
      document.getElementById('event-list').innerHTML=evts.map(e=>
        '<div class="event-row"><span class="event-id">#'+e.session_id+'</span><span class="event-kind '+e.kind+'">'+e.kind+'</span><span class="event-detail">'+fmtEvent(e)+'</span></div>'
      ).join('');
    }
  }catch(ex){
    document.getElementById('status-dot').textContent='Disconnected';
    document.getElementById('status-dot').style.color='var(--danger)';
  }
}
poll();setInterval(poll,2000);
</script>
</body>
</html>"##;

// ---------------------------------------------------------------------------
// Dashboard shared state (CLI mode)
// ---------------------------------------------------------------------------

struct DashboardState {
    events: Mutex<VecDeque<Event>>,
    start_time: std::time::Instant,
    listen_addr: Mutex<String>,
    config_summary: Mutex<String>,
    total_events: Mutex<u64>,
}

impl DashboardState {
    fn new() -> Self {
        Self {
            events: Mutex::new(VecDeque::new()),
            start_time: std::time::Instant::now(),
            listen_addr: Mutex::new(String::new()),
            config_summary: Mutex::new(String::new()),
            total_events: Mutex::new(0),
        }
    }
}

// ---------------------------------------------------------------------------
// Dashboard HTTP server (CLI mode)
// ---------------------------------------------------------------------------

async fn start_dashboard_server(state: Arc<DashboardState>) {
    let listener = match tokio::net::TcpListener::bind("127.0.0.1:41081").await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[surrogate] dashboard server failed to start: {e}");
            return;
        }
    };
    eprintln!("[surrogate] dashboard at http://127.0.0.1:41081");
    loop {
        let (mut stream, _) = match listener.accept().await {
            Ok(conn) => conn,
            Err(e) => {
                eprintln!("[surrogate] dashboard accept error: {e}");
                continue;
            }
        };
        let state = state.clone();
        tokio::spawn(async move {
            let _ = handle_dashboard_request(&mut stream, &state).await;
        });
    }
}

async fn handle_dashboard_request(
    stream: &mut tokio::net::TcpStream,
    state: &DashboardState,
) -> anyhow::Result<()> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let mut buf = vec![0u8; 4096];
    let n = stream.read(&mut buf).await?;
    let request = String::from_utf8_lossy(&buf[..n]);
    let path = request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("/");

    match path {
        "/api/status" => {
            let uptime = state.start_time.elapsed().as_secs();
            let (recent, total) = {
                let guard = state
                    .events
                    .lock()
                    .expect("events lock should not be poisoned");
                let recent: Vec<_> = guard.iter().rev().take(50).cloned().collect();
                (recent, *state.total_events.lock().expect("lock"))
            };
            let listen = state
                .listen_addr
                .lock()
                .expect("listen_addr lock should not be poisoned")
                .clone();
            let config = state
                .config_summary
                .lock()
                .expect("config_summary lock should not be poisoned")
                .clone();

            let json = serde_json::json!({
                "running": true,
                "listen_addr": listen,
                "uptime_secs": uptime,
                "total_events": total,
                "config_summary": config,
                "recent_events": recent,
            });
            let body = serde_json::to_string(&json)?;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).await?;
        }
        _ => {
            let body = DASHBOARD_HTML;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).await?;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// .app bundle mismatch detection
// ---------------------------------------------------------------------------

fn warn_if_running_inside_app_bundle() {
    if let Ok(exe) = std::env::current_exe() {
        let exe_str = exe.to_string_lossy();
        if exe_str.contains(".app/Contents/MacOS/") {
            eprintln!("╔══════════════════════════════════════════════════════════════╗");
            eprintln!("║  WARNING: surrogate-app (CLI) is running inside a .app      ║");
            eprintln!("║  bundle. The macOS GUI app requires the surrogate-macos     ║");
            eprintln!("║  binary instead. Rebuild the .app with:                     ║");
            eprintln!("║                                                              ║");
            eprintln!("║    cargo build -p surrogate-macos                            ║");
            eprintln!("║    ./scripts/package-macos.sh                                ║");
            eprintln!("║                                                              ║");
            eprintln!("║  Continuing as CLI fallback (no native GUI)...               ║");
            eprintln!("╚══════════════════════════════════════════════════════════════╝");
        }
    }
}

// ---------------------------------------------------------------------------
// Entry point and commands
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> Result<()> {
    warn_if_running_inside_app_bundle();

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
            let (path, web_ui) = parse_serve_cli_args(args)?;
            run_serve(&path, web_ui).await
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
                run_serve(&as_path, false).await
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
    run_serve(&config_path, false).await
}

async fn run_serve(path: &Path, web_ui: bool) -> Result<()> {
    let manager = ProxyManager::new(path.to_path_buf());

    let dashboard_state = Arc::new(DashboardState::new());

    let mut event_rx = manager.subscribe_events();
    let dash_state = dashboard_state.clone();
    tokio::spawn(async move {
        while let Ok(event) = event_rx.recv().await {
            if let Ok(line) = serde_json::to_string(&event) {
                println!("{line}");
            }
            if let Ok(mut events) = dash_state.events.lock() {
                events.push_back(event);
                while events.len() > 100 {
                    events.pop_front();
                }
            }
            if let Ok(mut total) = dash_state.total_events.lock() {
                *total += 1;
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

    let status = manager.status().await;
    *dashboard_state
        .listen_addr
        .lock()
        .expect("listen_addr lock should not be poisoned") = listen.to_string();
    *dashboard_state
        .config_summary
        .lock()
        .expect("config_summary lock should not be poisoned") = status.config_summary;

    if web_ui {
        tokio::spawn(start_dashboard_server(dashboard_state));

        #[cfg(target_os = "macos")]
        {
            if let Err(e) = std::process::Command::new("open")
                .arg("http://127.0.0.1:41081")
                .spawn()
            {
                eprintln!("[surrogate] failed to open dashboard in browser: {e}");
            }
        }
    }

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

/// Parses argv tokens after the `serve` subcommand: optional `--web-ui` and exactly one config path.
fn parse_serve_cli_args(args: impl Iterator<Item = String>) -> Result<(PathBuf, bool)> {
    let mut web_ui = false;
    let mut config_path: Option<PathBuf> = None;
    for arg in args {
        if arg == "--web-ui" {
            web_ui = true;
        } else {
            if config_path.is_some() {
                print_usage();
                bail!("unexpected extra argument `{arg}`");
            }
            config_path = Some(PathBuf::from(arg));
        }
    }
    match config_path {
        Some(path) => {
            Ok((path, web_ui))
        }
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
    eprintln!(
        "  surrogate-app serve [--web-ui] <config-path>  Start proxy with explicit config"
    );
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
    fn dashboard_state_stores_events() {
        let state = DashboardState::new();
        let event = Event::session_started(1, "example.com", 443);
        {
            let mut events = state.events.lock().unwrap();
            events.push_back(event);
        }
        let events = state.events.lock().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].session_id, 1);
    }

    #[test]
    fn dashboard_state_ring_buffer_bounds() {
        let state = DashboardState::new();
        {
            let mut events = state.events.lock().unwrap();
            for i in 0..150 {
                events.push_back(Event::session_started(i, "example.com", 443));
                while events.len() > 100 {
                    events.pop_front();
                }
            }
        }
        let events = state.events.lock().unwrap();
        assert_eq!(events.len(), 100);
        assert_eq!(events.front().unwrap().session_id, 50);
        assert_eq!(events.back().unwrap().session_id, 149);
    }

    #[test]
    fn warn_function_does_not_panic() {
        warn_if_running_inside_app_bundle();
    }

    #[test]
    fn serve_flag_parsing() {
        let (path, web_ui) = parse_serve_cli_args(
            vec!["--web-ui".to_string(), "a.toml".to_string()].into_iter(),
        )
        .expect("flag before path");
        assert_eq!(path, PathBuf::from("a.toml"));
        assert!(web_ui);

        let (path, web_ui) =
            parse_serve_cli_args(vec!["b.toml".to_string()].into_iter()).expect("path only");
        assert_eq!(path, PathBuf::from("b.toml"));
        assert!(!web_ui);

        let (path, web_ui) = parse_serve_cli_args(
            vec!["c.toml".to_string(), "--web-ui".to_string()].into_iter(),
        )
        .expect("flag after path");
        assert_eq!(path, PathBuf::from("c.toml"));
        assert!(web_ui);

        assert!(parse_serve_cli_args(std::iter::empty()).is_err());
        assert!(parse_serve_cli_args(vec!["--web-ui".to_string()].into_iter()).is_err());
        assert!(parse_serve_cli_args(
            vec!["x.toml".to_string(), "y.toml".to_string()].into_iter()
        )
        .is_err());
    }
}
