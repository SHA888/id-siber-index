//! Reviewer defect-escape ranking query
//!
//! Identifies reviewers with the highest defect-escape rates for use in
//! monthly meta-review audits. Helps pinpoint systemic review quality issues.

#![allow(dead_code)]

use anyhow::{Context, Result};
use sea_orm::{ColumnTrait, Database, DatabaseConnection, EntityTrait, QueryFilter};
use serde::Serialize;

use schema::entities::incident_correction;
use schema::entities::review_audit_log;

/// Arguments for reviewer ranking query
#[derive(Debug, Clone, Default)]
pub struct ReviewerRankingArgs {
    /// Target month for ranking (defaults to last month)
    pub month: Option<(u32, u32)>, // (year, month) as (2024, 5)
    /// Minimum acceptance count to include (default: 5)
    pub min_acceptances: Option<usize>,
    /// Limit results to top N reviewers (default: 20)
    pub limit: Option<usize>,
}

/// Reviewer quality metric
#[derive(Debug, Clone, Serialize)]
pub struct ReviewerMetric {
    pub reviewer_id: String,
    pub total_acceptances: usize,
    pub defect_escapes: usize,
    pub escape_rate_percent: f64,
    pub confidence_avg: f64,
}

/// Establish a database connection from the DATABASE_URL environment variable
async fn get_db() -> Result<DatabaseConnection> {
    let database_url =
        std::env::var("DATABASE_URL").context("DATABASE_URL environment variable not set")?;
    Database::connect(&database_url)
        .await
        .context("Failed to connect to database")
}

/// Query reviewer defect-escape rates
///
/// Returns a list of reviewers ranked by defect-escape rate (highest first).
/// Only includes reviewers with at least `min_acceptances` in the target period.
pub async fn query_reviewer_ranking(args: ReviewerRankingArgs) -> Result<Vec<ReviewerMetric>> {
    let db = get_db().await?;

    let min_acceptances = args.min_acceptances.unwrap_or(5);
    let limit = args.limit.unwrap_or(20);

    // Fetch all review_audit_log records with action="ACCEPTED"
    let accepted_reviews = review_audit_log::Entity::find()
        .filter(review_audit_log::Column::Action.eq("ACCEPTED"))
        .all(&db)
        .await
        .context("Failed to query accepted reviews")?;

    // Fetch all incident_correction records with their original_review_id
    let corrections = incident_correction::Entity::find()
        .all(&db)
        .await
        .context("Failed to query corrections")?;

    // Build a map: original_review_id -> correction
    let corrections_by_review_id: std::collections::HashMap<String, Vec<_>> = corrections
        .iter()
        .fold(std::collections::HashMap::new(), |mut map, corr| {
            map.entry(corr.original_review_id.to_string())
                .or_insert_with(Vec::new)
                .push(corr);
            map
        });

    // Group acceptances by reviewer
    let mut reviewer_stats: std::collections::HashMap<String, (usize, usize, f64)> =
        std::collections::HashMap::new();

    for review in &accepted_reviews {
        let escape_count = corrections_by_review_id
            .get(&review.id.to_string())
            .map(|v| v.len())
            .unwrap_or(0);

        let entry = reviewer_stats
            .entry(review.reviewer_id.clone())
            .or_insert((0, 0, 0.0));

        entry.0 += 1; // total acceptances
        entry.1 += escape_count; // defect escapes
        entry.2 += review.confidence_score.unwrap_or(0.5) as f64; // confidence sum
    }

    // Filter and calculate rates
    let mut metrics: Vec<ReviewerMetric> = reviewer_stats
        .into_iter()
        .filter(|(_, (acceptances, _, _))| *acceptances >= min_acceptances)
        .map(|(reviewer_id, (acceptances, escapes, confidence_sum))| {
            let escape_rate = if acceptances > 0 {
                (escapes as f64 / acceptances as f64) * 100.0
            } else {
                0.0
            };

            let confidence_avg = if acceptances > 0 {
                confidence_sum / acceptances as f64
            } else {
                0.0
            };

            ReviewerMetric {
                reviewer_id,
                total_acceptances: acceptances,
                defect_escapes: escapes,
                escape_rate_percent: escape_rate,
                confidence_avg,
            }
        })
        .collect();

    // Sort by escape rate (descending) then by confidence (ascending)
    metrics.sort_by(|a, b| {
        match b.escape_rate_percent.partial_cmp(&a.escape_rate_percent) {
            Some(std::cmp::Ordering::Equal) => a.confidence_avg.partial_cmp(&b.confidence_avg),
            other => other,
        }
        .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Limit results
    metrics.truncate(limit);

    Ok(metrics)
}

/// Display formatter for reviewer metrics
impl std::fmt::Display for ReviewerMetric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:<40} | Accepted: {:<4} | Escapes: {:<3} | Rate: {:<6.1}% | Conf: {:.2}",
            self.reviewer_id,
            self.total_acceptances,
            self.defect_escapes,
            self.escape_rate_percent,
            self.confidence_avg
        )
    }
}

/// Main entry point for reviewer ranking query
pub async fn run(args: ReviewerRankingArgs) -> Result<()> {
    println!("\n{}", "═".repeat(130));
    println!("  Reviewer Defect-Escape Ranking");
    println!("{}", "═".repeat(130));
    println!();

    let metrics = query_reviewer_ranking(args).await?;

    if metrics.is_empty() {
        println!("No reviewers found meeting minimum acceptance threshold.");
        return Ok(());
    }

    println!(
        "{:<40} | {:10} | {:10} | {:8} | {:10}",
        "Reviewer ID", "Accepted", "Escapes", "Rate %", "Avg Conf"
    );
    println!("{}", "─".repeat(130));

    for (i, metric) in metrics.iter().enumerate() {
        if i < 5 {
            print!("🔴 ");
        } else if i < 10 {
            print!("🟡 ");
        } else {
            print!("   ");
        }
        println!("{}", metric);
    }

    println!("\n{}", "═".repeat(130));
    println!("  Legend: 🔴 = Top 5 (critical), 🟡 = 6-10 (warning), blank = acceptable");
    println!("{}", "═".repeat(130));

    Ok(())
}
