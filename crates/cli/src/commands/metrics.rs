//! Metrics aggregation batch job
//!
//! Computes daily review metrics from audit logs and inserts into review_metrics table.
//! Metrics include: acceptance_rate, edit_rate, escalation_rate, mean_review_time_minutes.

#![allow(dead_code)]

use anyhow::{Context, Result};
use chrono::{NaiveDate, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, Database, DatabaseConnection, EntityTrait,
    PaginatorTrait, QueryFilter, QueryOrder, Set,
};
use tracing::info;

use schema::entities::incident;
use schema::entities::incident_correction;
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

/// Count defect escapes (corrections) reported for a given date
async fn count_daily_defects<C: ConnectionTrait>(db: &C, date: NaiveDate) -> Result<usize> {
    let start_of_day = date.and_hms_opt(0, 0, 0).unwrap();
    let end_of_day = date.and_hms_opt(23, 59, 59).unwrap();

    let count = incident_correction::Entity::find()
        .filter(incident_correction::Column::ReportedAt.gte(start_of_day))
        .filter(incident_correction::Column::ReportedAt.lte(end_of_day))
        .count(db)
        .await
        .context("Failed to count defect escapes")?;

    Ok(count as usize)
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
    date: NaiveDate,
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

    // Count defect escapes (incidents marked for correction on this date)
    let defect_escape_count = count_daily_defects(db, date).await?;

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

            let metrics = compute_metrics(&db, &logs, date).await?;
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

        let metrics = compute_metrics(&db, &logs, target_date).await?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::TransactionTrait;
    use uuid::Uuid;

    async fn with_transaction<F, Fut, T>(f: F) -> T
    where
        F: FnOnce(sea_orm::DatabaseTransaction) -> Fut,
        Fut: std::future::Future<Output = T>,
    {
        let db = get_db()
            .await
            .expect("Failed to get database connection for testing");
        let txn = db
            .begin()
            .await
            .expect("Failed to begin transaction for testing");
        f(txn).await
    }

    #[tokio::test]
    #[ignore]
    async fn test_count_daily_defects_zero_corrections() {
        with_transaction(|txn| async move {
            let test_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
            let count = count_daily_defects(&txn, test_date)
                .await
                .expect("Failed to count defects");

            assert_eq!(
                count, 0,
                "Should return 0 when no corrections exist for the date"
            );
        })
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_count_daily_defects_with_multiple_corrections() {
        with_transaction(|txn| async move {
            let test_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
            let naive_dt = test_date.and_hms_opt(12, 30, 45).unwrap();
            let correction_time = chrono::DateTime::<Utc>::from_naive_utc_and_offset(naive_dt, Utc);

            for i in 0..3 {
                let correction = incident_correction::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    incident_id: Set(Uuid::new_v4()),
                    original_review_id: Set(Uuid::new_v4()),
                    correction_type: Set("FACTUAL_ERROR".to_string()),
                    reported_by: Set(format!("tester-{}", i)),
                    description: Set(format!("Test correction {}", i)),
                    reported_at: Set(correction_time),
                    resolved_at: Set(None),
                    resolution_action: Set(None),
                };
                correction
                    .insert(&txn)
                    .await
                    .expect("Failed to insert test correction");
            }

            let count = count_daily_defects(&txn, test_date)
                .await
                .expect("Failed to count defects");

            assert_eq!(
                count, 3,
                "Should count exactly 3 corrections on the test date"
            );
        })
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_count_daily_defects_filters_by_date() {
        with_transaction(|txn| async move {
            let test_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
            let other_date = NaiveDate::from_ymd_opt(2024, 1, 14).unwrap();
            let test_time_naive = test_date.and_hms_opt(12, 0, 0).unwrap();
            let test_time =
                chrono::DateTime::<Utc>::from_naive_utc_and_offset(test_time_naive, Utc);
            let other_time_naive = other_date.and_hms_opt(12, 0, 0).unwrap();
            let other_time =
                chrono::DateTime::<Utc>::from_naive_utc_and_offset(other_time_naive, Utc);

            let correction_test = incident_correction::ActiveModel {
                id: Set(Uuid::new_v4()),
                incident_id: Set(Uuid::new_v4()),
                original_review_id: Set(Uuid::new_v4()),
                correction_type: Set("FACTUAL_ERROR".to_string()),
                reported_by: Set("tester-test".to_string()),
                description: Set("On test date".to_string()),
                reported_at: Set(test_time),
                resolved_at: Set(None),
                resolution_action: Set(None),
            };
            correction_test
                .insert(&txn)
                .await
                .expect("Failed to insert test correction");

            let correction_other = incident_correction::ActiveModel {
                id: Set(Uuid::new_v4()),
                incident_id: Set(Uuid::new_v4()),
                original_review_id: Set(Uuid::new_v4()),
                correction_type: Set("WRONG_SECTOR".to_string()),
                reported_by: Set("tester-other".to_string()),
                description: Set("On other date".to_string()),
                reported_at: Set(other_time),
                resolved_at: Set(None),
                resolution_action: Set(None),
            };
            correction_other
                .insert(&txn)
                .await
                .expect("Failed to insert other correction");

            let count = count_daily_defects(&txn, test_date)
                .await
                .expect("Failed to count defects");

            assert_eq!(
                count, 1,
                "Should only count corrections on the specified date"
            );
        })
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_count_daily_defects_day_boundary() {
        with_transaction(|txn| async move {
            let test_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
            let end_of_day_naive = test_date.and_hms_opt(23, 59, 59).unwrap();
            let end_of_day =
                chrono::DateTime::<Utc>::from_naive_utc_and_offset(end_of_day_naive, Utc);
            let before_day_naive = end_of_day_naive - chrono::Duration::minutes(1);
            let before_day =
                chrono::DateTime::<Utc>::from_naive_utc_and_offset(before_day_naive, Utc);

            let correction_in = incident_correction::ActiveModel {
                id: Set(Uuid::new_v4()),
                incident_id: Set(Uuid::new_v4()),
                original_review_id: Set(Uuid::new_v4()),
                correction_type: Set("FACTUAL_ERROR".to_string()),
                reported_by: Set("tester-in".to_string()),
                description: Set("Within day".to_string()),
                reported_at: Set(end_of_day),
                resolved_at: Set(None),
                resolution_action: Set(None),
            };
            correction_in
                .insert(&txn)
                .await
                .expect("Failed to insert in-day correction");

            let correction_before = incident_correction::ActiveModel {
                id: Set(Uuid::new_v4()),
                incident_id: Set(Uuid::new_v4()),
                original_review_id: Set(Uuid::new_v4()),
                correction_type: Set("WRONG_SECTOR".to_string()),
                reported_by: Set("tester-before".to_string()),
                description: Set("Before day".to_string()),
                reported_at: Set(before_day),
                resolved_at: Set(None),
                resolution_action: Set(None),
            };
            correction_before
                .insert(&txn)
                .await
                .expect("Failed to insert before-day correction");

            let count = count_daily_defects(&txn, test_date)
                .await
                .expect("Failed to count defects");

            assert_eq!(
                count, 1,
                "Should only count corrections within the exact day boundaries"
            );
        })
        .await;
    }
}
