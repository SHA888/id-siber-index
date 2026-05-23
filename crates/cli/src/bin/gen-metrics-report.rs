//! Generate markdown metrics report from review_metrics table
//!
//! Reads from the review_metrics table and generates a human-readable METRICS.md report
//! with daily summary, weekly trend, and month-to-date statistics.

use anyhow::{Context, Result};
use chrono::{Datelike, Duration, Local, NaiveDate, Utc};
use sea_orm::{ColumnTrait, Database, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};
use std::fs;

use schema::entities::review_metrics;

/// Establish database connection from DATABASE_URL env var
async fn get_db() -> Result<DatabaseConnection> {
    let database_url =
        std::env::var("DATABASE_URL").context("DATABASE_URL environment variable not set")?;
    Database::connect(&database_url)
        .await
        .context("Failed to connect to database")
}

/// Metrics snapshot for a specific date
#[derive(Clone, Debug)]
struct MetricsSnapshot {
    date: NaiveDate,
    incidents_reviewed: i32,
    acceptance_rate: f64,
    edit_rate: f64,
    escalation_rate: f64,
    mean_review_time_minutes: f64,
    defect_escape_count: i32,
}

impl From<review_metrics::Model> for MetricsSnapshot {
    fn from(model: review_metrics::Model) -> Self {
        MetricsSnapshot {
            date: model.date,
            incidents_reviewed: model.incidents_reviewed,
            acceptance_rate: model.acceptance_rate,
            edit_rate: model.edit_rate,
            escalation_rate: model.escalation_rate,
            mean_review_time_minutes: model.mean_review_time_minutes,
            defect_escape_count: model.defect_escape_count,
        }
    }
}

/// Fetch metrics for the last N days
async fn fetch_recent_metrics(db: &DatabaseConnection, days: i64) -> Result<Vec<MetricsSnapshot>> {
    let cutoff_date = (Utc::now() - Duration::days(days)).date_naive();

    let metrics = review_metrics::Entity::find()
        .filter(review_metrics::Column::Date.gte(cutoff_date))
        .order_by_desc(review_metrics::Column::Date)
        .all(db)
        .await
        .context("Failed to query review_metrics")?;

    Ok(metrics.into_iter().map(MetricsSnapshot::from).collect())
}

/// Calculate aggregate metrics over a date range
fn aggregate_metrics(snapshots: &[MetricsSnapshot]) -> Option<MetricsSnapshot> {
    if snapshots.is_empty() {
        return None;
    }

    let total_incidents: i32 = snapshots.iter().map(|m| m.incidents_reviewed).sum();
    let total_defects: i32 = snapshots.iter().map(|m| m.defect_escape_count).sum();

    // Weighted average rates (by incidents reviewed)
    let total_weight: i32 = total_incidents;
    if total_weight == 0 {
        return None;
    }

    let avg_acceptance_rate = snapshots
        .iter()
        .map(|m| m.acceptance_rate * m.incidents_reviewed as f64)
        .sum::<f64>()
        / total_weight as f64;

    let avg_edit_rate = snapshots
        .iter()
        .map(|m| m.edit_rate * m.incidents_reviewed as f64)
        .sum::<f64>()
        / total_weight as f64;

    let avg_escalation_rate = snapshots
        .iter()
        .map(|m| m.escalation_rate * m.incidents_reviewed as f64)
        .sum::<f64>()
        / total_weight as f64;

    let avg_review_time = snapshots
        .iter()
        .map(|m| m.mean_review_time_minutes * m.incidents_reviewed as f64)
        .sum::<f64>()
        / total_weight as f64;

    Some(MetricsSnapshot {
        date: snapshots.first().unwrap().date,
        incidents_reviewed: total_incidents,
        acceptance_rate: avg_acceptance_rate,
        edit_rate: avg_edit_rate,
        escalation_rate: avg_escalation_rate,
        mean_review_time_minutes: avg_review_time,
        defect_escape_count: total_defects,
    })
}

/// Format a metrics snapshot as a table row
fn format_metrics_row(m: &MetricsSnapshot) -> String {
    format!(
        "| {} | {} | {:.1}% | {:.1}% | {:.1}% | {:.1} min | {} |",
        m.date,
        m.incidents_reviewed,
        m.acceptance_rate,
        m.edit_rate,
        m.escalation_rate,
        m.mean_review_time_minutes,
        m.defect_escape_count
    )
}

/// Generate markdown report from metrics
async fn generate_report(db: &DatabaseConnection) -> Result<String> {
    let now = Local::now();
    let today = now.date_naive();

    // Fetch last 30 days of metrics
    let recent_metrics = fetch_recent_metrics(db, 30).await?;

    // Extract today's metrics if available
    let today_metrics = recent_metrics.iter().find(|m| m.date == today);

    // Get 7-day trend (excluding today if incomplete)
    let week_ago = today - Duration::days(7);
    let seven_day_metrics: Vec<_> = recent_metrics
        .iter()
        .filter(|m| m.date >= week_ago && m.date < today)
        .cloned()
        .collect();

    // Get month-to-date stats
    let month_start = today.with_day(1).unwrap();
    let mtd_metrics: Vec<_> = recent_metrics
        .iter()
        .filter(|m| m.date >= month_start && m.date < today)
        .cloned()
        .collect();

    // Generate markdown
    let mut report = String::new();
    report.push_str("# Incident Review Metrics\n\n");
    report.push_str(&format!(
        "**Last Updated:** {} UTC\n\n",
        now.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
    ));

    // Today's metrics section
    report.push_str("## Today's Metrics\n\n");
    if let Some(m) = today_metrics {
        report.push_str("| Metric | Value |\n");
        report.push_str("|--------|-------|\n");
        report.push_str(&format!("| Date | {} |\n", m.date));
        report.push_str(&format!(
            "| Incidents Reviewed | {} |\n",
            m.incidents_reviewed
        ));
        report.push_str(&format!(
            "| Acceptance Rate | {:.1}% |\n",
            m.acceptance_rate
        ));
        report.push_str(&format!("| Edit Rate | {:.1}% |\n", m.edit_rate));
        report.push_str(&format!(
            "| Escalation Rate | {:.1}% |\n",
            m.escalation_rate
        ));
        report.push_str(&format!(
            "| Mean Review Time | {:.1} minutes |\n",
            m.mean_review_time_minutes
        ));
        report.push_str(&format!("| Defect Escapes | {} |\n", m.defect_escape_count));
    } else {
        report.push_str("_No metrics available for today yet._\n");
    }
    report.push('\n');

    // 7-day trend section
    report.push_str("## 7-Day Trend\n\n");
    if !seven_day_metrics.is_empty() {
        report.push_str(
            "| Date | Incidents | Acceptance | Edit | Escalation | Avg Time | Defects |\n",
        );
        report.push_str(
            "|------|-----------|------------|------|------------|----------|----------|\n",
        );
        for m in seven_day_metrics.iter().rev() {
            report.push_str(&format_metrics_row(m));
            report.push('\n');
        }
    } else {
        report.push_str("_Insufficient data for trend analysis._\n");
    }
    report.push('\n');

    // Month-to-date summary section
    report.push_str("## Month-to-Date Summary\n\n");
    if let Some(agg) = aggregate_metrics(&mtd_metrics) {
        report.push_str("| Metric | Value |\n");
        report.push_str("|--------|-------|\n");
        report.push_str(&format!("| Days Aggregated | {} |\n", mtd_metrics.len()));
        report.push_str(&format!(
            "| Total Incidents | {} |\n",
            agg.incidents_reviewed
        ));
        report.push_str(&format!(
            "| Avg Acceptance Rate | {:.1}% |\n",
            agg.acceptance_rate
        ));
        report.push_str(&format!("| Avg Edit Rate | {:.1}% |\n", agg.edit_rate));
        report.push_str(&format!(
            "| Avg Escalation Rate | {:.1}% |\n",
            agg.escalation_rate
        ));
        report.push_str(&format!(
            "| Avg Review Time | {:.1} minutes |\n",
            agg.mean_review_time_minutes
        ));
        report.push_str(&format!(
            "| Total Defect Escapes | {} |\n",
            agg.defect_escape_count
        ));
    } else {
        report.push_str("_Insufficient data for aggregation._\n");
    }
    report.push('\n');

    // Alerts section
    report.push_str("## Alerts & Observations\n\n");
    let mut alerts = Vec::new();

    if let Some(m) = today_metrics {
        if m.escalation_rate > 20.0 {
            alerts.push(format!(
                "⚠️ High escalation rate today: {:.1}%",
                m.escalation_rate
            ));
        }
        if m.defect_escape_count > 2 {
            alerts.push(format!(
                "⚠️ Multiple defects escaped review today: {}",
                m.defect_escape_count
            ));
        }
        if m.mean_review_time_minutes > 30.0 {
            alerts.push(format!(
                "⚠️ Slow review times today: {:.1} min avg",
                m.mean_review_time_minutes
            ));
        }
    }

    if alerts.is_empty() {
        report.push_str("✓ No alerts.\n");
    } else {
        for alert in alerts {
            report.push_str(&format!("- {}\n", alert));
        }
    }
    report.push('\n');

    // Footer
    report.push_str("---\n\n");
    report.push_str(
        "*This report is automatically generated by GitHub Actions (daily at 2am UTC).*\n",
    );

    Ok(report)
}

#[tokio::main]
async fn main() -> Result<()> {
    let db = get_db().await?;
    let report = generate_report(&db).await?;

    // Write to METRICS.md
    fs::write("METRICS.md", report).context("Failed to write METRICS.md")?;

    println!("✓ METRICS.md generated successfully");
    Ok(())
}
