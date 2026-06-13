use anyhow::{Result, anyhow};
use clap::Parser;
use seslogin::db::Handler as _;
use seslogin::nitc_export::{self, NitcConfig};
use seslogin::request_metrics::{self, RequestMetrics};
use seslogin::sqs_dispatch;
use std::sync::Arc;

#[derive(Parser, Debug)]
#[command(about = "Sync attendance periods to NITC (Non-Incident Training/Activity) events")]
struct Cli {
    /// Assign a single period to its NITC event (Phase 1 only).
    #[arg(long, conflicts_with_all = ["event_id", "bump_period_id", "bump_event_id"])]
    period_id: Option<String>,

    /// Run Phase 2 sync on a single NITC event by ID (uses its current DB version).
    #[arg(long, conflicts_with_all = ["period_id", "bump_period_id", "bump_event_id"])]
    event_id: Option<String>,

    /// Bump a period's version and enqueue a Phase 1 SQS message.
    #[arg(long, conflicts_with_all = ["period_id", "event_id", "bump_event_id"])]
    bump_period_id: Option<String>,

    /// Bump a NITC event's version and enqueue a Phase 2 SQS message.
    #[arg(long, conflicts_with_all = ["period_id", "event_id", "bump_period_id"])]
    bump_event_id: Option<String>,

    /// Scan all NITC-enabled locations for unsynced periods and enqueue them for export.
    #[arg(long, conflicts_with_all = ["period_id", "event_id", "bump_period_id", "bump_event_id"])]
    backfill: bool,

    /// Only process this location ID (for use with --backfill).
    #[arg(long)]
    location_id: Option<String>,

    /// Print what would happen without making any changes.
    #[arg(long, default_value_t = false)]
    dry_run: bool,

    /// Re-sync periods/events even if they are already synced.
    #[arg(long, default_value_t = false)]
    force: bool,

    #[arg(long)]
    ses_api_base_url: Option<String>,

    #[arg(long)]
    ses_api_key: Option<String>,

    #[arg(long)]
    db_prefix: Option<String>,

    #[arg(long)]
    nitc_queue_url: Option<String>,

    #[arg(long)]
    max_retries: Option<usize>,
}

fn build_config(cli: &Cli) -> Result<(NitcConfig, String)> {
    let ses_api_base_url = cli
        .ses_api_base_url
        .clone()
        .or_else(|| std::env::var("SES_API_BASE_URL").ok())
        .ok_or_else(|| anyhow!("SES_API_BASE_URL is required"))?;
    let ses_api_key = cli
        .ses_api_key
        .clone()
        .or_else(|| std::env::var("SES_API_KEY").ok())
        .ok_or_else(|| anyhow!("SES_API_KEY is required"))?;
    let db_prefix = cli
        .db_prefix
        .clone()
        .or_else(|| std::env::var("DB_PREFIX").ok())
        .ok_or_else(|| anyhow!("DB_PREFIX is required"))?;
    let nitc_queue_url = cli
        .nitc_queue_url
        .clone()
        .or_else(|| std::env::var("NITC_EXPORT_QUEUE_URL").ok())
        .ok_or_else(|| anyhow!("NITC_EXPORT_QUEUE_URL is required"))?;
    let max_retries = cli
        .max_retries
        .or_else(|| {
            std::env::var("NITC_MAX_RETRIES")
                .ok()
                .and_then(|v| v.parse().ok())
        })
        .unwrap_or(3);

    Ok((
        NitcConfig {
            dry_run: cli.dry_run,
            force: cli.force,
            ses_api_base_url,
            ses_api_key,
            nitc_queue_url,
            max_retries,
        },
        db_prefix,
    ))
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    dotenvy::from_filename(".env").ok();
    dotenvy::from_filename(".env.secret").ok();

    let cli = Cli::parse();
    let (config, db_prefix) = build_config(&cli)?;
    let clients = nitc_export::make_dynamodb_clients(&config, db_prefix).await?;

    let metrics = Arc::new(RequestMetrics::default());
    request_metrics::METRICS
        .scope(metrics.clone(), async move {
            if let Some(ref period_id) = cli.period_id {
                let result = nitc_export::assign_period(period_id, &config, &clients).await?;
                match result {
                    nitc_export::PeriodAssignOutcome::Assigned(event_id) => {
                        println!("period {} → assigned to event {}", period_id, event_id)
                    }
                    nitc_export::PeriodAssignOutcome::AlreadySynced => {
                        println!("period {} → already synced", period_id)
                    }
                    nitc_export::PeriodAssignOutcome::Skipped(reason) => {
                        println!("period {} → skipped ({})", period_id, reason)
                    }
                }
                return anyhow::Ok(());
            }

            if let Some(ref event_id) = cli.event_id {
                let outcome =
                    nitc_export::sync_nitc_event(event_id, None, &config, &clients).await?;
                match outcome {
                    nitc_export::EventSyncOutcome::Synced(ses_id) => println!(
                        "event {} → synced https://beacon.ses.nsw.gov.au/nitc/{}",
                        event_id, ses_id
                    ),
                    nitc_export::EventSyncOutcome::Skipped(reason) => {
                        println!("event {} → skipped ({})", event_id, reason)
                    }
                    other => println!("event {} → {:?}", event_id, other),
                }
                return anyhow::Ok(());
            }

            if let Some(ref period_id) = cli.bump_period_id {
                let new_version = clients.db.bump_period_version(period_id).await?;
                sqs_dispatch::enqueue_period_nitc_export(
                    &clients.sqs.client,
                    &clients.sqs.queue_url,
                    period_id,
                )
                .await?;
                println!(
                    "period {} → bumped to version {}, Phase 1 SQS message enqueued",
                    period_id, new_version
                );
                return anyhow::Ok(());
            }

            if let Some(ref event_id) = cli.bump_event_id {
                let new_version = clients.db.bump_nitc_event_version(event_id).await?;
                sqs_dispatch::enqueue_nitc_event_export(
                    &clients.sqs.client,
                    &clients.sqs.queue_url,
                    event_id,
                    new_version,
                )
                .await?;
                println!(
                    "event {} → bumped to version {}, Phase 2 SQS message enqueued",
                    event_id, new_version
                );
                return anyhow::Ok(());
            }

            if cli.backfill {
                let stats = nitc_export::backfill_unsynced_periods(
                    cli.location_id.as_deref(),
                    &config,
                    &clients,
                )
                .await?;
                println!(
                    "locations checked: {}, periods enqueued: {}, periods already synced: {}, periods skipped (no NITC category): {}",
                    stats.locations_checked, stats.periods_enqueued, stats.periods_already_synced, stats.periods_skipped_no_nitc_category
                );
                return anyhow::Ok(());
            }

            eprintln!(
                "Specify one of --period-id, --event-id, --bump-period-id, --bump-event-id, or --backfill"
            );
            std::process::exit(1);
        })
        .await?;

    tracing::info!(
        "total rru={:.1} wru={:.1}",
        metrics.read_units(),
        metrics.write_units(),
    );

    Ok(())
}
