use std::env;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{bail, Context, Result};
use clap::{ArgAction, Args, Parser, Subcommand, ValueEnum};
use contracts::{RadarEnvelope, PROTOCOL_VERSION};
use core_engine::{Engine, EngineConfig, GameDataProvider, MockProvider};
use dump_refresh::refresh_from_files;
use dump_runtime::{resolve_active_dump, DumpRuntimeConfig};
use tiny_http::{Header, Method, Request, Response, Server, StatusCode};
use tracing::{info, warn};

const LOCALHOST_HOST: &str = "127.0.0.1";
const DEFAULT_UI_PORT: u16 = 4173;
const DEFAULT_UI_PORT_FALLBACK_SPAN: u16 = 10;

#[derive(Parser, Debug)]
#[command(author, version, about = "cs2_webradar_extreme desktop runtime")]
#[command(args_conflicts_with_subcommands = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<RootCommand>,
    #[command(flatten)]
    run: RunArgs,
}

#[derive(Subcommand, Debug)]
enum RootCommand {
    Run(RunArgs),
    Dump {
        #[command(subcommand)]
        command: DumpCommand,
    },
}

#[derive(Subcommand, Debug)]
enum DumpCommand {
    Refresh(RefreshArgs),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum UiMode {
    Desktop,
    Browser,
}

#[derive(Args, Debug, Clone)]
struct RunArgs {
    #[arg(long, default_value_t = 25)]
    rate_hz: u16,
    #[arg(long)]
    overlay: bool,
    #[arg(long)]
    click_through: bool,
    #[arg(long)]
    stream_json: bool,
    #[arg(long)]
    max_ticks: Option<u64>,
    #[arg(long, value_enum, default_value_t = UiMode::Desktop)]
    ui_mode: UiMode,
    #[arg(long, default_value_t = DEFAULT_UI_PORT)]
    ui_port: u16,
    #[arg(long, default_value_t = DEFAULT_UI_PORT_FALLBACK_SPAN)]
    ui_port_fallback_span: u16,
    #[arg(long, default_value_t = true, action = ArgAction::Set)]
    open_browser: bool,
}

impl Default for RunArgs {
    fn default() -> Self {
        Self {
            rate_hz: 25,
            overlay: false,
            click_through: false,
            stream_json: false,
            max_ticks: None,
            ui_mode: UiMode::Desktop,
            ui_port: DEFAULT_UI_PORT,
            ui_port_fallback_span: DEFAULT_UI_PORT_FALLBACK_SPAN,
            open_browser: true,
        }
    }
}

#[derive(Args, Debug)]
struct RefreshArgs {
    #[arg(long)]
    output: PathBuf,
    #[arg(long)]
    build_number: u32,
    #[arg(long)]
    binary: Option<PathBuf>,
    #[arg(long)]
    schema: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();
    let Cli { command, run } = Cli::parse();
    let command = command.unwrap_or_else(|| {
        info!("no subcommand provided; starting default run mode");
        RootCommand::Run(run)
    });

    match command {
        RootCommand::Run(args) => run_command(args).await,
        RootCommand::Dump {
            command: DumpCommand::Refresh(args),
        } => refresh_command(args),
    }
}

fn refresh_command(args: RefreshArgs) -> Result<()> {
    let report = refresh_from_files(
        &args.output,
        args.binary.as_deref(),
        args.schema.as_deref(),
        args.build_number,
    )?;

    info!(
        "dump refresh complete: path={}, offsets={}, classes={}",
        report.output_dir.display(),
        report.offsets_count,
        report.schema_classes
    );

    Ok(())
}

async fn run_command(args: RunArgs) -> Result<()> {
    if args.ui_mode == UiMode::Browser {
        if args.overlay || args.click_through {
            warn!("overlay flags are ignored when --ui-mode browser is selected");
        }
        return host_browser_ui(&args);
    }

    if args.overlay {
        if cfg!(target_os = "windows") {
            info!(
                "overlay mode enabled (click-through: {})",
                args.click_through
            );
        } else {
            warn!("overlay mode requested on unsupported platform, running in desktop mode");
        }
    }

    let provider = MockProvider::new();
    let expected_build = provider.current_build_number();

    let dump_config = DumpRuntimeConfig {
        app_id: "cs2_webradar_extreme".to_string(),
        embedded_dir: embedded_dump_dir(),
        user_override_dir: None,
    };

    let active_dump = resolve_active_dump(&dump_config, expected_build)
        .or_else(|_| resolve_active_dump(&dump_config, None))
        .context("unable to resolve dump pack")?;

    for warning in &active_dump.warnings {
        warn!("{warning}");
    }

    let tick_interval_ms = (1000_u16 / args.rate_hz.max(1)).max(1);
    let config = EngineConfig {
        tick_interval_ms,
        interpolation_window_ms: 100,
        invalid_frame_threshold: 8,
    };

    let engine = Engine::new(config, provider, active_dump);
    stream_snapshots(engine, args).await
}

fn host_browser_ui(args: &RunArgs) -> Result<()> {
    let ui_dist = resolve_ui_dist_dir();
    if let Some(path) = &ui_dist {
        info!("serving browser UI assets from {}", path.display());
    } else {
        warn!("ui/dist was not found; serving fallback startup page");
    }

    let (server, port) = bind_localhost_server(args.ui_port, args.ui_port_fallback_span)?;
    let url = format!("http://{LOCALHOST_HOST}:{port}");

    info!("browser-hosted UI mode active at {url}");
    println!("Browser UI: {url}");

    if args.open_browser {
        match webbrowser::open(&url) {
            Ok(_) => info!("opened browser at {url}"),
            Err(error) => warn!("unable to auto-open browser: {error}"),
        }
    }

    serve_browser_requests(server, ui_dist.as_deref())
}

fn bind_localhost_server(preferred_port: u16, fallback_span: u16) -> Result<(Server, u16)> {
    let mut last_error: Option<String> = None;

    for offset in 0..=fallback_span {
        let Some(candidate_port) = preferred_port.checked_add(offset) else {
            break;
        };

        match Server::http((LOCALHOST_HOST, candidate_port)) {
            Ok(server) => {
                if offset > 0 {
                    warn!(
                        "default UI port {} was unavailable; using fallback port {}",
                        preferred_port, candidate_port
                    );
                }
                return Ok((server, candidate_port));
            }
            Err(error) => {
                last_error = Some(error.to_string());
                warn!(
                    "unable to bind UI server on {}:{}, trying next fallback port",
                    LOCALHOST_HOST, candidate_port
                );
            }
        }
    }

    bail!(
        "unable to bind localhost UI server in range {}-{} ({})",
        preferred_port,
        preferred_port.saturating_add(fallback_span),
        last_error.unwrap_or_else(|| "unknown error".to_string())
    )
}

fn serve_browser_requests(server: Server, ui_dist: Option<&Path>) -> Result<()> {
    for request in server.incoming_requests() {
        if let Err(error) = respond_to_request(request, ui_dist) {
            warn!("browser UI request failed: {error}");
        }
    }

    Ok(())
}

fn respond_to_request(request: Request, ui_dist: Option<&Path>) -> Result<()> {
    if request.method() != &Method::Get {
        return send_response(
            request,
            StatusCode(405),
            b"Method Not Allowed".to_vec(),
            "text/plain; charset=utf-8",
        );
    }

    let request_path = request.url().split('?').next().unwrap_or("/");
    let Some(relative_path) = sanitize_request_path(request_path) else {
        return send_response(
            request,
            StatusCode(400),
            b"Bad Request".to_vec(),
            "text/plain; charset=utf-8",
        );
    };

    if let Some(root) = ui_dist {
        return send_response_for_asset(request, root, &relative_path);
    }

    send_response(
        request,
        StatusCode(200),
        fallback_browser_page().into_bytes(),
        "text/html; charset=utf-8",
    )
}

fn send_response_for_asset(request: Request, root: &Path, relative_path: &Path) -> Result<()> {
    let mut file_path = if relative_path.as_os_str().is_empty() {
        root.join("index.html")
    } else {
        root.join(relative_path)
    };

    if file_path.is_dir() {
        file_path = file_path.join("index.html");
    }

    if file_path.is_file() {
        let bytes = fs::read(&file_path)
            .with_context(|| format!("failed to read UI asset {}", file_path.display()))?;
        return send_response(
            request,
            StatusCode(200),
            bytes,
            content_type_for_path(&file_path),
        );
    }

    if relative_path.extension().is_none() {
        let index_path = root.join("index.html");
        if index_path.is_file() {
            let bytes = fs::read(&index_path).with_context(|| {
                format!("failed to read UI index file {}", index_path.display())
            })?;
            return send_response(
                request,
                StatusCode(200),
                bytes,
                "text/html; charset=utf-8",
            );
        }
    }

    send_response(
        request,
        StatusCode(404),
        b"Not Found".to_vec(),
        "text/plain; charset=utf-8",
    )
}

fn send_response(request: Request, status: StatusCode, body: Vec<u8>, content_type: &str) -> Result<()> {
    let content_type_header = Header::from_bytes("Content-Type", content_type)
        .map_err(|_| anyhow::anyhow!("invalid content-type header: {content_type}"))?;

    let response = Response::from_data(body)
        .with_status_code(status)
        .with_header(content_type_header);

    request
        .respond(response)
        .context("failed to write UI HTTP response")?;

    Ok(())
}

fn sanitize_request_path(raw_path: &str) -> Option<PathBuf> {
    let trimmed = raw_path.trim_start_matches('/');
    if trimmed.is_empty() {
        return Some(PathBuf::new());
    }

    let mut normalized = PathBuf::new();
    for component in Path::new(trimmed).components() {
        match component {
            Component::Normal(part) => normalized.push(part),
            Component::CurDir => {}
            _ => return None,
        }
    }

    Some(normalized)
}

fn content_type_for_path(path: &Path) -> &'static str {
    match path.extension().and_then(|ext| ext.to_str()).unwrap_or_default() {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "ico" => "image/x-icon",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        _ => "application/octet-stream",
    }
}

fn fallback_browser_page() -> String {
    "<!doctype html><html><head><meta charset=\"utf-8\"><title>CS2 Webradar Extreme</title></head><body><h1>CS2 Webradar Extreme</h1><p>UI assets were not found.</p><p>Build the frontend with <code>npm --prefix ./ui run build</code> and run again, or set <code>CS2_WEBRADAR_UI_DIST</code> to a folder containing index.html.</p></body></html>".to_string()
}

fn resolve_ui_dist_dir() -> Option<PathBuf> {
    let mut candidates = Vec::new();

    if let Ok(path) = env::var("CS2_WEBRADAR_UI_DIST") {
        candidates.push(PathBuf::from(path));
    }

    candidates.push(manifest_join("../../../ui/dist"));
    candidates.push(PathBuf::from("ui/dist"));

    if let Ok(exe_path) = env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            candidates.push(exe_dir.join("ui").join("dist"));
            for ancestor in exe_dir.ancestors().take(6) {
                candidates.push(ancestor.join("ui").join("dist"));
            }
        }
    }

    candidates
        .into_iter()
        .find(|path| path.join("index.html").is_file())
}

async fn stream_snapshots(mut engine: Engine<MockProvider>, args: RunArgs) -> Result<()> {
    let tick_interval = Duration::from_millis(u64::from((1000_u16 / args.rate_hz.max(1)).max(1)));
    let mut interval = tokio::time::interval(tick_interval);
    let mut emitted = 0_u64;

    loop {
        interval.tick().await;
        let snapshot = engine.next_snapshot();
        let envelope = RadarEnvelope {
            protocol_version: PROTOCOL_VERSION,
            emitted_at_ms: now_ms(),
            snapshot,
        };

        emitted += 1;
        if args.stream_json {
            println!("{}", serde_json::to_string(&envelope)?);
        } else {
            println!(
                "tick={} map={} players={} bomb={}",
                envelope.snapshot.tick,
                envelope.snapshot.match_state.map_name,
                envelope.snapshot.players.len(),
                envelope.snapshot.bomb.is_some()
            );
        }

        if args.max_ticks.is_some_and(|limit| emitted >= limit) {
            break;
        }
    }

    Ok(())
}

fn embedded_dump_dir() -> PathBuf {
    manifest_join("../../../data/dumps/embedded")
}

fn manifest_join(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis() as u64)
}
