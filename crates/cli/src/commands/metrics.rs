//! Metrics aggregation batch job
//!
//! Computes daily review metrics from audit logs and inserts into review_metrics table.
//! Metrics include: acceptance_rate, edit_rate, escalation_rate, mean_review_time_minutes.

#![allow(dead_code)]

use anyhow::{Context, Result};
use chrono::{NaiveDate, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Database, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder, Set,
};
use tracing::info;

use schema::entities::incident;
use schema::entities::review_audit_log::{self, Entity as ReviewAuditLogEntity};
use schema::entities::review_metrics::ActiveModel as ReviewMetricsActiveModel;

/// Metrics aggregation arguments
#[derive(Debug, Clone, Default)]
pub struct MetricsArgs {
    /// Target date for aggregation (defaults to yesterday)
    pub date: Option<NaiveDate>,
    /// Dry run: print metrics without committing
    pub dry_run: bool,
    /// Look back N days (for backfill)
    pub backfill_days: Option<usize>,
}

/// Establish a database connection from the DATABASE_URL environment variable
async fn get_db() -> Result<DatabaseConnection> {
    let database_url =
        std::env::var("DATABASE_URL").context("DATABASE_URL environment variable not set")?;
    Database::connect(&database_url)
        .await
        .context("Failed to connect to database")
}

/// Fetch all review audit logs for a given date
async fn fetch_daily_logs(
    db: &DatabaseConnection,
    date: NaiveDate,
) -> Result<Vec<review_audit_log::Model>> {
    let start_of_day = date.and_hms_opt(0, 0, 0).unwrap();
    let end_of_day = date.and_hms_opt(23, 59, 59).unwrap();

    let logs = ReviewAuditLogEntity::find()
        .filter(review_audit_log::Column::ReviewedAt.gte(start_of_day))
        .filter(review_audit_log::Column::ReviewedAt.lte(end_of_day))
        .order_by_asc(review_audit_log::Column::ReviewedAt)
        .all(db)
        .await
        .context("Failed to query audit logs")?;

    Ok(logs)
}

/// Compute review metrics from audit logs
struct DailyMetrics {
    pub incidents_reviewed: usize,
    pub acceptance_rate: f64,
    pub edit_rate: f64,
    pub escalation_rate: f64,
    pub mean_review_time_minutes: f64,
    pub defect_escape_count: usize,
}

async fn compute_metrics(
    db: &DatabaseConnection,
    logs: &[review_audit_log::Model],
) -> Result<DailyMetrics> {
    if logs.is_empty() {
        return Ok(DailyMetrics {
            incidents_reviewed: 0,
            acceptance_rate: 0.0,
            edit_rate: 0.0,
            escalation_rate: 0.0,
            mean_review_time_minutes: 0.0,
            defect_escape_count: 0,
        });
    }

    // Count actions
    let mut accepted_count = 0usize;
    let mut rejected_count = 0usize;
    let mut edited_count = 0usize;
    let mut escalated_count = 0usize;
    let mut total_review_minutes = 0i64;
    let mut incidents_with_review = std::collections::HashSet::new();

    for log in logs {
        incidents_with_review.insert(log.incident_id);

        match log.action.as_str() {
            "ACCEPTED" => accepted_count += 1,
            "REJECTED" => rejected_count += 1,
            "EDITED" => edited_count += 1,
            "ESCALATED" => escalated_count += 1,
            _ => {}
        }

        // Estimate review time from incident creation to review
        if let Ok(Some(incident)) = incident::Entity::find_by_id(log.incident_id).one(db).await {
            let duration = log.reviewed_at.timestamp() - incident.created_at.timestamp();
            total_review_minutes += duration / 60; // Convert to minutes
        }
    }

    let incidents_reviewed = incidents_with_review.len();
    let total_decisions = accepted_count + rejected_count + escalated_count;
    let total_with_edits = total_decisions + edited_count;

    let acceptance_rate = if total_decisions > 0 {
        (accepted_count as f64 / total_decisions as f64) * 100.0
    } else {
        0.0
    };

    let edit_rate = if total_with_edits > 0 {
        (edited_count as f64 / total_with_edits as f64) * 100.0
    } else {
        0.0
    };

    let escalation_rate = if total_decisions > 0 {
        (escalated_count as f64 / total_decisions as f64) * 100.0
    } else {
        0.0
    };

    let mean_review_time_minutes = if !logs.is_empty() {
        total_review_minutes as f64 / logs.len() as f64
    } else {
        0.0
    };

    // Count defect escapes (incidents marked for correction)
    let defect_escape_count = 0; // TODO: Query incident_correction table when implemented

    Ok(DailyMetrics {
        incidents_reviewed,
        acceptance_rate,
        edit_rate,
        escalation_rate,
        mean_review_time_minutes,
        defect_escape_count,
    })
}

/// Save metrics to database
async fn save_metrics(
    db: &DatabaseConnection,
    date: NaiveDate,
    metrics: &DailyMetrics,
    dry_run: bool,
) -> Result<()> {
    let created_at = Utc::now();

    if dry_run {
        println!("\n[DRY RUN] Would insert metrics for {}:", date);
        println!("  Incidents reviewed:     {}", metrics.incidents_reviewed);
        println!("  Acceptance rate:        {:.1}%", metrics.acceptance_rate);
        println!("  Edit rate:              {:.1}%", metrics.edit_rate);
        println!("  Escalation rate:        {:.1}%", metrics.escalation_rate);
        println!(
            "  Mean review time:       {:.1} minutes",
            metrics.mean_review_time_minutes
        );
        println!("  Defect escape count:    {}", metrics.defect_escape_count);
        return Ok(());
    }

    let active_model = ReviewMetricsActiveModel {
        date: Set(date),
        incidents_reviewed: Set(metrics.incidents_reviewed as i32),
        acceptance_rate: Set(metrics.acceptance_rate),
        edit_rate: Set(metrics.edit_rate),
        escalation_rate: Set(metrics.escalation_rate),
        mean_review_time_minutes: Set(metrics.mean_review_time_minutes),
        defect_escape_count: Set(metrics.defect_escape_count as i32),
        created_at: Set(created_at),
        updated_at: Set(created_at),
    };

    active_model
        .insert(db)
        .await
        .context("Failed to insert metrics")?;

    info!("Metrics aggregated for {}", date);
    Ok(())
}

/// Main entry point for metrics aggregation
pub async fn run(args: MetricsArgs) -> Result<()> {
    info!("Starting metrics aggregation...");

    let db = get_db().await?;

    let target_date = args.date.unwrap_or_else(|| {
        // Default to yesterday
        (Utc::now() - chrono::Duration::days(1)).date_naive()
    });

    println!("\n{}", "═".repeat(70));
    println!("  Metrics Aggregation for {}", target_date);
    println!("{}", "═".repeat(70));

    if let Some(backfill_days) = args.backfill_days {
        // Backfill mode: process N days
        for days_ago in (0..backfill_days).rev() {
            let date = target_date - chrono::Duration::days(days_ago as i64);
            let logs = fetch_daily_logs(&db, date).await?;

            if logs.is_empty() {
                println!("\n  {} — No review activity", date);
                continue;
            }

            let metrics = compute_metrics(&db, &logs).await?;
            save_metrics(&db, date, &metrics, args.dry_run).await?;

            println!("\n  {} — Aggregated {} reviews", date, logs.len());
            println!("    Acceptance rate: {:.1}%", metrics.acceptance_rate);
            println!("    Edit rate:       {:.1}%", metrics.edit_rate);
            println!("    Escalation rate: {:.1}%", metrics.escalation_rate);
            println!(
                "    Mean review time: {:.1} min",
                metrics.mean_review_time_minutes
            );
        }
    } else {
        // Single day mode
        let logs = fetch_daily_logs(&db, target_date).await?;

        if logs.is_empty() {
            println!("\n  No review activity on {}.", target_date);
            return Ok(());
        }

        let metrics = compute_metrics(&db, &logs).await?;
        save_metrics(&db, target_date, &metrics, args.dry_run).await?;

        println!("\n  {} incidents reviewed", metrics.incidents_reviewed);
        println!("  Acceptance rate:  {:.1}%", metrics.acceptance_rate);
        println!("  Edit rate:        {:.1}%", metrics.edit_rate);
        println!("  Escalation rate:  {:.1}%", metrics.escalation_rate);
        println!(
            "  Mean review time: {:.1} minutes",
            metrics.mean_review_time_minutes
        );
    }

    println!("\n{}", "═".repeat(70));
    if args.dry_run {
        println!("  [DRY RUN] No changes committed");
    } else {
        println!("  ✓ Metrics aggregated successfully");
    }
    println!("{}", "═".repeat(70));

    Ok(())
}
