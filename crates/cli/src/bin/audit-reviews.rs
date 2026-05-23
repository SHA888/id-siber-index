//! Monthly meta-review audit script
//!
//! Samples 10% of review_audit_log entries and audits:
//! - Justification substantiveness (confidence/action alignment + presence)
//! - Confidence score distribution (stats + anomalies)
//! - Escalation legitimacy (checks against trigger rules)
//!
//! Generates a markdown report with findings and recommendations.

use anyhow::{Context, Result};
use chrono::Local;
use rand::seq::SliceRandom;
use sea_orm::{Database, DatabaseConnection, EntityTrait};
use std::collections::BTreeMap;

use schema::entities::review_audit_log;

/// Establish database connection from DATABASE_URL env var
async fn get_db() -> Result<DatabaseConnection> {
    let database_url =
        std::env::var("DATABASE_URL").context("DATABASE_URL environment variable not set")?;
    Database::connect(&database_url)
        .await
        .context("Failed to connect to database")
}

/// Audit result for a single review
#[derive(Clone, Debug)]
struct ReviewAudit {
    id: String,
    action: String,
    confidence_score: f32,
    has_justification: bool,
    is_escalated: bool,
    issues: Vec<String>,
}

/// Aggregate audit statistics
#[derive(Clone, Debug)]
struct AuditStats {
    total_reviews: usize,
    sample_size: usize,
    reviews_with_issues: usize,
    avg_confidence: f32,
    confidence_histogram: BTreeMap<String, usize>,
    escalation_count: usize,
    escalation_issues: usize,
    reviews_without_justification: usize,
    low_confidence_accepts: usize,
}

/// Audit a single review for quality issues
fn audit_review(review: &review_audit_log::Model) -> ReviewAudit {
    let mut issues = Vec::new();

    let confidence = review.confidence_score.unwrap_or(0.0);
    let has_justification = review.justification.is_some()
        && !review
            .justification
            .as_ref()
            .unwrap_or(&String::new())
            .trim()
            .is_empty();
    let is_escalated = review.action == "ESCALATED";

    // Check 1: Missing or empty justification
    if !has_justification {
        issues.push("No justification provided for review decision".to_string());
    }

    // Check 2: High confidence rejections need strong justification
    if review.action == "REJECTED" && confidence >= 0.8 && !has_justification {
        issues.push("High confidence rejection without justification".to_string());
    }

    // Check 3: Low confidence accepts are suspicious
    if review.action == "ACCEPTED" && confidence < 0.3 {
        issues.push("Low confidence acceptance (< 0.3)".to_string());
    }

    // Check 4: Escalated without low confidence
    if is_escalated && confidence >= 0.6 {
        issues.push("Escalation with moderate-to-high confidence (unusual)".to_string());
    }

    // Check 5: Low confidence accept that should have been escalated
    if review.action == "ACCEPTED" && confidence < 0.3 && !is_escalated {
        issues.push("Low confidence acceptance that should have been escalated".to_string());
    }

    // Check 6: Edited reviews should have reasonable confidence
    if review.action == "EDITED" && confidence > 0.9 {
        issues.push("Very high confidence edit (edits usually indicate uncertainty)".to_string());
    }

    ReviewAudit {
        id: review.id.to_string(),
        action: review.action.clone(),
        confidence_score: confidence,
        has_justification,
        is_escalated,
        issues,
    }
}

/// Compute audit statistics from sampled reviews
fn compute_stats(audits: &[ReviewAudit], total_reviews: usize) -> AuditStats {
    let sample_size = audits.len();
    let mut issues_count = 0;
    let mut confidence_sum = 0.0;
    let mut escalation_count = 0;
    let mut escalation_issues = 0;
    let mut no_justification_count = 0;
    let mut low_conf_accepts = 0;
    let mut histogram = BTreeMap::new();

    for audit in audits {
        if !audit.issues.is_empty() {
            issues_count += 1;
        }

        confidence_sum += audit.confidence_score as f64;

        // Histogram buckets
        let bucket = match (audit.confidence_score * 10.0) as usize {
            0 => "0.0-0.1",
            1 => "0.1-0.2",
            2 => "0.2-0.3",
            3 => "0.3-0.4",
            4 => "0.4-0.5",
            5 => "0.5-0.6",
            6 => "0.6-0.7",
            7 => "0.7-0.8",
            8 => "0.8-0.9",
            _ => "0.9-1.0",
        };
        *histogram.entry(bucket.to_string()).or_insert(0) += 1;

        if audit.is_escalated {
            escalation_count += 1;
            if !audit.issues.is_empty() {
                escalation_issues += 1;
            }
        }

        if !audit.has_justification {
            no_justification_count += 1;
        }

        if audit.action == "ACCEPTED" && audit.confidence_score < 0.3 {
            low_conf_accepts += 1;
        }
    }

    let avg_confidence = if sample_size > 0 {
        (confidence_sum / sample_size as f64) as f32
    } else {
        0.0
    };

    AuditStats {
        total_reviews,
        sample_size,
        reviews_with_issues: issues_count,
        avg_confidence,
        confidence_histogram: histogram,
        escalation_count,
        escalation_issues,
        reviews_without_justification: no_justification_count,
        low_confidence_accepts: low_conf_accepts,
    }
}

/// Format audit report as markdown
fn format_report(audits: &[ReviewAudit], stats: &AuditStats) -> String {
    let mut report = String::new();
    let now = Local::now();

    report.push_str("# Monthly Review Audit Report\n\n");
    report.push_str(&format!(
        "**Generated:** {}\n\n",
        now.format("%Y-%m-%d %H:%M:%S")
    ));

    // Executive summary
    report.push_str("## Executive Summary\n\n");
    report.push_str(&format!(
        "- **Total Reviews in Period:** {}\n",
        stats.total_reviews
    ));
    report.push_str(&format!("- **Sample Size (10%):** {}\n", stats.sample_size));
    report.push_str(&format!(
        "- **Reviews with Issues:** {} ({:.1}%)\n",
        stats.reviews_with_issues,
        (stats.reviews_with_issues as f64 / stats.sample_size as f64) * 100.0
    ));
    report.push_str(&format!(
        "- **Average Confidence Score:** {:.2}\n\n",
        stats.avg_confidence
    ));

    // Confidence distribution
    report.push_str("## Confidence Score Distribution\n\n");
    report.push_str("| Range | Count | % |\n");
    report.push_str("|-------|-------|----|\n");
    let mut total = 0;
    for (bucket, count) in &stats.confidence_histogram {
        let pct = (*count as f64 / stats.sample_size as f64) * 100.0;
        report.push_str(&format!("| {} | {} | {:.1}% |\n", bucket, count, pct));
        total += count;
    }
    report.push_str(&format!(
        "\n_Mean: {:.2}, Distribution shows {} reviews above 0.8 confidence_\n\n",
        stats.avg_confidence,
        stats
            .confidence_histogram
            .iter()
            .filter(|(k, _)| k.as_str() >= "0.8-0.9")
            .map(|(_, v)| v)
            .sum::<usize>()
    ));

    // Escalation audit
    report.push_str("## Escalation Audit\n\n");
    report.push_str(&format!(
        "- **Total Escalations:** {}\n",
        stats.escalation_count
    ));
    report.push_str(&format!(
        "- **Escalations with Issues:** {}\n",
        stats.escalation_issues
    ));
    report.push_str(&format!(
        "- **Reviews without Justification:** {}\n",
        stats.reviews_without_justification
    ));
    report.push_str(&format!(
        "- **Low Confidence Accepts:** {}\n\n",
        stats.low_confidence_accepts
    ));

    if stats.low_confidence_accepts > 0 {
        report.push_str(
            "⚠️ **Alert:** Found low-confidence acceptances that may need escalation review.\n\n",
        );
    }

    // Issues found
    report.push_str("## Issues Found\n\n");
    if stats.reviews_with_issues == 0 {
        report.push_str("✓ No quality issues detected in sample.\n\n");
    } else {
        report.push_str(&format!(
            "{} reviews had quality concerns:\n\n",
            stats.reviews_with_issues
        ));

        // Group issues by type
        let mut issue_types: BTreeMap<String, usize> = BTreeMap::new();
        for audit in audits {
            for issue in &audit.issues {
                *issue_types.entry(issue.clone()).or_insert(0) += 1;
            }
        }

        for (issue_type, count) in issue_types {
            report.push_str(&format!("- **{}:** {} review(s)\n", issue_type, count));
        }

        // Sample problematic reviews (up to 3)
        let problematic: Vec<_> = audits.iter().filter(|a| !a.issues.is_empty()).collect();
        if !problematic.is_empty() {
            report.push_str("\n### Sample Issues\n\n");
            for (i, audit) in problematic.iter().take(3).enumerate() {
                report.push_str(&format!(
                    "{}. Review {}: {} (confidence: {:.2})\n",
                    i + 1,
                    &audit.id[..8],
                    audit.action,
                    audit.confidence_score
                ));
                for issue in &audit.issues {
                    report.push_str(&format!("   - {}\n", issue));
                }
            }
            report.push('\n');
        }
    }

    // Recommendations
    report.push_str("## Recommendations\n\n");
    if stats.low_confidence_accepts > 0 {
        report.push_str(
            "1. **Review low-confidence acceptances** — Consider escalation for decisions below 0.3 confidence\n",
        );
    }
    if stats.reviews_with_issues > (stats.sample_size / 20) {
        report.push_str("2. **Review quality training** — Consider reviewer training if >5% of reviews have issues\n");
    }
    if stats.avg_confidence < 0.6 {
        report.push_str("3. **Investigate low baseline confidence** — Average confidence <0.6 may indicate reviewer uncertainty\n");
    }
    report.push_str("4. **Regular audits** — Continue monthly audits to track quality trends\n\n");

    report.push_str("---\n\n");
    report.push_str(
        "_This audit was automatically generated. For details, contact the review team._\n",
    );

    report
}

#[tokio::main]
async fn main() -> Result<()> {
    let db = get_db().await?;

    // Fetch all reviews and sample
    let all_reviews = review_audit_log::Entity::find()
        .all(&db)
        .await
        .context("Failed to query review_audit_log")?;

    let total_reviews = all_reviews.len();

    if total_reviews == 0 {
        eprintln!("No reviews found in review_audit_log");
        println!("# Monthly Review Audit Report\n\nNo reviews to audit yet.\n");
        return Ok(());
    }

    // Sample 10% (or at least 1)
    let sample_size = std::cmp::max(1, (total_reviews as f64 * 0.1).ceil() as usize);
    let mut rng = rand::thread_rng();
    let sampled: Vec<_> = all_reviews
        .choose_multiple(&mut rng, sample_size)
        .cloned()
        .collect();

    eprintln!(
        "Auditing {} out of {} reviews ({:.1}%)",
        sample_size,
        total_reviews,
        (sample_size as f64 / total_reviews as f64) * 100.0
    );

    // Audit each sampled review
    let audits: Vec<_> = sampled.iter().map(audit_review).collect();

    // Compute statistics
    let stats = compute_stats(&audits, total_reviews);

    // Format and print report
    let report = format_report(&audits, &stats);
    println!("{}", report);

    Ok(())
}
