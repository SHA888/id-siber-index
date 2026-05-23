//! Mark incident defect command
//!
//! Records defects that escaped the review process into the incident_correction table.
//! Used to track and measure defect-escape rate for quality metrics.

#![allow(dead_code)]

use anyhow::{Context, Result, bail};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, Database, DatabaseConnection, EntityTrait, Set};
use std::str::FromStr;
use tracing::info;
use uuid::Uuid;

use schema::entities::incident;
use schema::entities::incident_correction::{self, CorrectionType};
use schema::entities::review_audit_log;

/// Mark defect command arguments
#[derive(Debug, Clone, Default)]
pub struct DefectArgs {
    /// Incident UUID that has a defect
    pub incident_id: Option<Uuid>,
    /// Type of correction (FACTUAL_ERROR, WRONG_SECTOR, WRONG_ATTACK_TYPE, DUPLICATE_MERGE, OTHER)
    pub correction_type: Option<String>,
    /// Description of the defect
    pub description: Option<String>,
    /// Original review audit log ID that failed to catch the defect
    pub original_review_id: Option<Uuid>,
    /// Reporter identifier (defaults to IDSIBER_REVIEWER_ID env var)
    pub reported_by: Option<String>,
    /// Dry run: print defect record without committing
    pub dry_run: bool,
}

/// Establish a database connection from the DATABASE_URL environment variable
async fn get_db() -> Result<DatabaseConnection> {
    let database_url =
        std::env::var("DATABASE_URL").context("DATABASE_URL environment variable not set")?;
    Database::connect(&database_url)
        .await
        .context("Failed to connect to database")
}

/// Get or prompt for reporter identifier
async fn get_reporter_id(provided: Option<String>) -> Result<String> {
    if let Some(id) = provided {
        return Ok(id);
    }

    if let Ok(id) = std::env::var("IDSIBER_REVIEWER_ID") {
        return Ok(id);
    }

    bail!(
        "Reporter ID is required. Set IDSIBER_REVIEWER_ID env var or pass --reported-by. \
         This should be the user/agent who discovered the defect."
    );
}

/// Validate correction type
fn validate_correction_type(correction_type: &str) -> Result<CorrectionType> {
    CorrectionType::from_str(correction_type).map_err(|_| {
        anyhow::anyhow!(
            "Invalid correction type '{}'. Must be one of: FACTUAL_ERROR, WRONG_SECTOR, \
             WRONG_ATTACK_TYPE, DUPLICATE_MERGE, OTHER",
            correction_type
        )
    })
}

/// Record a defect for an incident
async fn record_defect(
    db: &DatabaseConnection,
    incident_id: Uuid,
    correction_type: &str,
    reported_by: &str,
    original_review_id: Uuid,
    description: &str,
    dry_run: bool,
) -> Result<()> {
    // Validate incident exists
    let incident = incident::Entity::find_by_id(incident_id)
        .one(db)
        .await
        .context("Failed to query incident")?
        .ok_or_else(|| anyhow::anyhow!("Incident {} not found", incident_id))?;

    // Validate correction type
    let corr_type = validate_correction_type(correction_type)?;

    // Validate original review exists
    let review = review_audit_log::Entity::find_by_id(original_review_id)
        .one(db)
        .await
        .context("Failed to query review audit log")?
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Original review {} not found. Cannot link defect to non-existent review.",
                original_review_id
            )
        })?;

    if dry_run {
        println!("\n[DRY RUN] Would record defect for incident:");
        println!("  Organization: {}", incident.org_name);
        println!("  Incident ID: {}", incident_id);
        println!("  Defect Type: {}", corr_type);
        println!("  Description: {}", description);
        println!("  Reported by: {}", reported_by);
        println!("  Linked to review: {}", original_review_id);
        println!("    (Review action: {})", review.action);
        return Ok(());
    }

    let now = Utc::now();
    let active_model = incident_correction::ActiveModel {
        id: Set(Uuid::new_v4()),
        incident_id: Set(incident_id),
        correction_type: Set(corr_type.to_string()),
        reported_by: Set(reported_by.to_string()),
        reported_at: Set(now),
        original_review_id: Set(original_review_id),
        description: Set(description.to_string()),
        resolved_at: Set(None),
        resolution_action: Set(None),
    };

    active_model
        .insert(db)
        .await
        .context("Failed to record defect")?;

    info!(
        "Defect recorded for incident {} ({}): {}",
        incident_id, corr_type, description
    );

    Ok(())
}

/// Main entry point for marking defects
pub async fn run(args: DefectArgs) -> Result<()> {
    info!("Recording incident defect...");

    // Validate required arguments
    let incident_id = args.incident_id.ok_or_else(|| {
        anyhow::anyhow!("--incident-id is required. Provide a UUID of the incident with a defect.")
    })?;

    let correction_type = args.correction_type.ok_or_else(|| {
        anyhow::anyhow!(
            "--correction-type is required. Must be one of: \
             FACTUAL_ERROR, WRONG_SECTOR, WRONG_ATTACK_TYPE, DUPLICATE_MERGE, OTHER"
        )
    })?;

    let description = args.description.ok_or_else(|| {
        anyhow::anyhow!(
            "--description is required. Provide a brief description of the defect \
             (e.g., 'Data breach should be marked as CRITICAL', 'Sector should be Healthcare not Finance')"
        )
    })?;

    let original_review_id = args.original_review_id.ok_or_else(|| {
        anyhow::anyhow!(
            "--original-review-id is required. This links the defect to the review action \
             that failed to catch it. Use 'isi review --audit-trail <incident-id>' to find the UUID."
        )
    })?;

    let reported_by = get_reporter_id(args.reported_by).await?;

    let db = get_db().await?;

    println!("\n{}", "═".repeat(70));
    println!("  Record Incident Defect");
    println!("{}", "═".repeat(70));

    record_defect(
        &db,
        incident_id,
        &correction_type,
        &reported_by,
        original_review_id,
        &description,
        args.dry_run,
    )
    .await?;

    println!("\n{}", "═".repeat(70));
    if args.dry_run {
        println!("  [DRY RUN] Defect record preview complete");
    } else {
        println!("  ✓ Defect recorded successfully");
        println!("  This defect will be included in the defect-escape rate metrics");
    }
    println!("{}", "═".repeat(70));

    Ok(())
}
