use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use contracts::{RadarEnvelope, PROTOCOL_VERSION};
use core_engine::{Engine, EngineConfig, GameDataProvider, MockProvider};
use dump_refresh::refresh_from_files;
use dump_runtime::{resolve_active_dump, DumpRuntimeConfig};
use tokio::sync::broadcast;
use tracing::{info, warn};

#[derive(Parser, Debug)]
#[command(author, version, about = "cs2_webradar_extreme desktop runtime")]
struct Cli {
    #[command(subcommand)]
    command: RootCommand,
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

#[derive(Args, Debug)]
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
    let cli = Cli::parse();

    match cli.command {
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

    let mut provider = MockProvider::new();
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

async fn stream_snapshots(mut engine: Engine<MockProvider>, args: RunArgs) -> Result<()> {
    let (tx, mut rx) = broadcast::channel::<RadarEnvelope>(64);
    let tick_interval = Duration::from_millis(u64::from((1000_u16 / args.rate_hz.max(1)).max(1)));

    let sender = tx.clone();
    let max_ticks = args.max_ticks;
    tokio::spawn(async move {
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

            let _ = sender.send(envelope);
            emitted += 1;
            if max_ticks.is_some_and(|limit| emitted >= limit) {
                break;
            }
        }
    });

    let mut consumed = 0_u64;
    while let Ok(envelope) = rx.recv().await {
        consumed += 1;
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

        if args.max_ticks.is_some_and(|limit| consumed >= limit) {
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
