//! Interactive review command for unverified incidents
//!
//! This command provides an interactive CLI to review, edit, accept, or reject
//! unverified incident records. It supports both interactive and batch modes.
//!
//! Every review action is logged to `review_audit_log` per REVIEW_SKILLS.md
//! principles 2.2 (citation) and 2.5 (signing).

use anyhow::{bail, Context, Result};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Database, DatabaseConnection, EntityTrait, QueryFilter, Set,
};
use serde_json::json;
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing::{info, warn};
use uuid::Uuid;

use schema::entities::incident::{self, Entity as IncidentEntity, Model as IncidentModel};
use schema::entities::review_audit_log::{self, Entity as ReviewAuditLogEntity, ReviewAction};

/// Review command arguments
#[derive(Debug, Clone, Default)]
pub struct ReviewArgs {
    /// Run in non-interactive batch mode
    pub batch: bool,
    /// Auto-accept incidents (batch mode only)
    pub auto_accept: bool,
    /// Auto-reject incidents (batch mode only)
    pub auto_reject: bool,
    /// Maximum number of records to process
    pub limit: Option<usize>,
    /// Filter by sector
    pub sector: Option<String>,
    /// Filter by attack type
    pub attack_type: Option<String>,
    /// Filter by source type
    pub source_type: Option<String>,
    /// Reviewer identifier (user or agent name)
    pub reviewer_id: Option<String>,
    /// Justification for the review action (batch mode)
    pub justification: Option<String>,
    /// Confidence score 0.0–1.0 (batch mode)
    pub confidence: Option<f32>,
    /// Force action without justification (rejections only)
    pub force: bool,
    /// Dry run: print actions without committing
    pub dry_run: bool,
    /// Maximum batch size (default 50)
    pub max_batch_size: Option<usize>,
    /// Query audit trail for a specific incident (UUID)
    pub audit_trail: Option<Uuid>,
}

/// Review context holds the reviewer identity for the session
#[derive(Debug, Clone)]
struct ReviewContext {
    reviewer_id: String,
    dry_run: bool,
}

/// Establish a database connection from the DATABASE_URL environment variable
async fn get_db() -> Result<DatabaseConnection> {
    let database_url =
        std::env::var("DATABASE_URL").context("DATABASE_URL environment variable not set")?;
    Database::connect(&database_url)
        .await
        .context("Failed to connect to database")
}

/// Get or prompt for reviewer identity
async fn get_reviewer_id(provided: Option<String>) -> Result<String> {
    if let Some(id) = provided {
        return Ok(id);
    }

    // Try to read from cached config
    if let Ok(id) = std::env::var("IDSIBER_REVIEWER_ID") {
        println!("Using reviewer ID from environment: {}", id);
        return Ok(id);
    }

    // Prompt interactively
    print!("Enter reviewer ID (or set IDSIBER_REVIEWER_ID env var): ");
    let input = read_line().await?;
    if input.is_empty() {
        bail!("Reviewer ID is required. Set IDSIBER_REVIEWER_ID or pass --reviewer-id.");
    }
    Ok(input)
}

/// Fetch unverified incidents matching the given filters
async fn fetch_unverified(
    db: &DatabaseConnection,
    args: &ReviewArgs,
) -> Result<Vec<IncidentModel>> {
    let mut query = IncidentEntity::find().filter(incident::Column::Verified.eq(false));

    if let Some(sector) = &args.sector {
        query = query.filter(incident::Column::OrgSector.eq(sector.as_str()));
    }

    if let Some(attack_type) = &args.attack_type {
        query = query.filter(incident::Column::AttackType.eq(attack_type.as_str()));
    }

    if let Some(source_type) = &args.source_type {
        query = query.filter(incident::Column::SourceType.eq(source_type.as_str()));
    }

    let mut results = query
        .all(db)
        .await
        .context("Failed to query unverified incidents")?;

    // Sort by creation date (oldest first for backfill)
    results.sort_by_key(|a| a.created_at);

    let limit = args.limit.or(args.max_batch_size).unwrap_or(usize::MAX);
    if results.len() > limit {
        results.truncate(limit);
    }

    Ok(results)
}

/// Fetch audit trail for a specific incident
async fn fetch_audit_trail(
    db: &DatabaseConnection,
    incident_id: Uuid,
) -> Result<Vec<review_audit_log::Model>> {
    ReviewAuditLogEntity::find()
        .filter(review_audit_log::Column::IncidentId.eq(incident_id))
        .order_by_desc(review_audit_log::Column::ReviewedAt)
        .all(db)
        .await
        .context("Failed to query audit trail")
}

/// Display audit trail for an incident
fn display_audit_trail(incident: &IncidentModel, entries: &[review_audit_log::Model]) {
    println!("\n{}", "═".repeat(70));
    println!("  Audit Trail for Incident {}", incident.id);
    println!("  Organization: {}", incident.org_name);
    println!("{}", "═".repeat(70));

    if entries.is_empty() {
        println!("  No review history found.");
        return;
    }

    for (idx, entry) in entries.iter().enumerate() {
        println!("\n  Entry {}:", idx + 1);
        println!("    Reviewer:     {}", entry.reviewer_id);
        println!("    Action:       {}", entry.action);
        println!(
            "    Reviewed At:  {}",
            entry.reviewed_at.format("%Y-%m-%d %H:%M:%S %Z")
        );
        if let Some(score) = entry.confidence_score {
            println!("    Confidence:   {:.2}", score);
        }
        if let Some(justification) = &entry.justification {
            println!("    Justification: {}", justification);
        }
    }
    println!("\n{}", "═".repeat(70));
}

/// Display an incident in a formatted way
fn display_incident(incident: &IncidentModel, index: usize, total: usize) {
    println!("\n{}", "─".repeat(70));
    println!("  Incident {} of {}", index + 1, total);
    println!("{}", "─".repeat(70));
    println!("  ID:              {}", incident.id);
    println!("  Organization:    {}", incident.org_name);
    println!("  Sector:          {}", incident.org_sector);
    println!(
        "  Incident Date:   {}",
        incident.incident_date.format("%Y-%m-%d")
    );
    println!(
        "  Disclosure Date: {}",
        incident.disclosure_date.format("%Y-%m-%d")
    );
    println!("  Attack Type:     {}", incident.attack_type);
    println!(
        "  Source:          {} ({})",
        incident.source_type, incident.source_url
    );
    if let Some(notes) = &incident.notes {
        println!("  Notes:           {}", notes);
    }
    if let Some(actor) = &incident.actor_alias {
        println!("  Actor Alias:     {}", actor);
    }
    if let Some(group) = &incident.actor_group {
        println!("  Actor Group:     {}", group);
    }
    println!("{}", "─".repeat(70));
}

/// Display available actions
fn display_actions() {
    println!("\n  Actions:");
    println!("    [a] Accept  - Mark as verified and save");
    println!("    [r] Reject  - Delete this incident");
    println!("    [e] Edit    - Modify fields interactively");
    println!("    [s] Skip    - Leave unchanged, go to next");
    println!("    [q] Quit    - Stop reviewing");
    print!("\n  Choice: ");
}

/// Read a line from stdin asynchronously
async fn read_line() -> Result<String> {
    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .await
        .context("Failed to read input")?;
    Ok(line.trim().to_lowercase())
}

/// Prompt for a new value, returning None if user enters empty string
async fn prompt_value(prompt: &str, current: &str) -> Result<Option<String>> {
    println!("\n  {}: {}", prompt, current);
    print!("  New value (or press Enter to keep): ");
    let input = read_line().await?;
    if input.is_empty() {
        Ok(None)
    } else {
        Ok(Some(input))
    }
}

/// Prompt for an optional value
async fn prompt_optional(prompt: &str, current: &Option<String>) -> Result<Option<String>> {
    let current_str = current.as_deref().unwrap_or("(none)");
    println!("\n  {}: {}", prompt, current_str);
    print!("  New value (or 'clear' to remove, Enter to keep): ");
    let input = read_line().await?;
    if input.is_empty() {
        Ok(None) // keep current
    } else if input == "clear" {
        Ok(Some(String::new())) // signal to clear
    } else {
        Ok(Some(input))
    }
}

/// Prompt for justification text
async fn prompt_justification(required: bool) -> Result<Option<String>> {
    if required {
        print!("\n  Justification (required): ");
    } else {
        print!("\n  Justification (optional, press Enter to skip): ");
    }
    let input = read_line().await?;
    if input.is_empty() && required {
        bail!("Justification is required for this action.");
    }
    if input.is_empty() {
        Ok(None)
    } else {
        Ok(Some(input))
    }
}

/// Prompt for confidence score
async fn prompt_confidence() -> Result<f32> {
    println!("\n  Confidence level:");
    println!("    [1] Low    (0.25) — uncertain, needs verification");
    println!("    [2] Medium (0.50) — reasonably confident (default)");
    println!("    [3] High   (0.90) — highly confident");
    println!("    [c] Custom — enter exact value 0.0–1.0");
    print!("\n  Choice [2]: ");

    let choice = read_line().await?;
    match choice.as_str() {
        "1" | "low" | "l" => Ok(0.25),
        "" | "2" | "medium" | "m" => Ok(0.50),
        "3" | "high" | "h" => Ok(0.90),
        "c" | "custom" => {
            print!("  Enter confidence value (0.0–1.0): ");
            let val_str = read_line().await?;
            let val: f32 = val_str.parse().context("Invalid confidence value")?;
            if !(0.0..=1.0).contains(&val) {
                bail!("Confidence must be between 0.0 and 1.0");
            }
            Ok(val)
        }
        _ => {
            println!("  Unknown choice, defaulting to Medium (0.50).");
            Ok(0.50)
        }
    }
}

/// Build a JSON snapshot of an incident model
fn snapshot_incident(incident: &IncidentModel) -> serde_json::Value {
    json!({
        "id": incident.id,
        "org_name": incident.org_name,
        "org_sector": incident.org_sector,
        "incident_date": incident.incident_date.to_string(),
        "disclosure_date": incident.disclosure_date.to_string(),
        "attack_type": incident.attack_type,
        "data_categories": incident.data_categories,
        "record_count_estimate": incident.record_count_estimate,
        "financial_impact_idr": incident.financial_impact_idr,
        "actor_alias": incident.actor_alias,
        "actor_group": incident.actor_group,
        "source_url": incident.source_url,
        "source_type": incident.source_type,
        "verified": incident.verified,
        "notes": incident.notes,
    })
}

/// Log a review action to the audit trail
async fn log_review_action(
    db: &DatabaseConnection,
    ctx: &ReviewContext,
    incident_id: Uuid,
    action: ReviewAction,
    justification: Option<String>,
    confidence_score: Option<f32>,
    prior_status: Option<serde_json::Value>,
    post_status: Option<serde_json::Value>,
) -> Result<()> {
    if ctx.dry_run {
        println!(
            "  [DRY RUN] Would log audit: {} by {} (confidence: {:?})",
            action, ctx.reviewer_id, confidence_score
        );
        return Ok(());
    }

    let active_model = review_audit_log::ActiveModel {
        id: Set(Uuid::new_v4()),
        incident_id: Set(incident_id),
        reviewer_id: Set(ctx.reviewer_id.clone()),
        action: Set(action.to_string()),
        reviewed_at: Set(chrono::Utc::now().into()),
        justification: Set(justification),
        confidence_score: Set(confidence_score),
        prior_status: Set(prior_status.map(Into::into)),
        post_status: Set(post_status.map(Into::into)),
    };

    active_model
        .insert(db)
        .await
        .context("Failed to write review audit log")?;

    Ok(())
}

/// Accept an incident (mark as verified)
async fn accept_incident(
    db: &DatabaseConnection,
    ctx: &ReviewContext,
    incident: &IncidentModel,
    justification: Option<String>,
    confidence: Option<f32>,
) -> Result<()> {
    let prior = snapshot_incident(incident);

    if ctx.dry_run {
        println!(
            "  [DRY RUN] Would accept incident {} (confidence: {:?})",
            incident.id, confidence
        );
        log_review_action(
            db,
            ctx,
            incident.id,
            ReviewAction::Accepted,
            justification.clone(),
            confidence,
            Some(prior.clone()),
            Some(prior),
        )
        .await?;
        return Ok(());
    }

    let mut active_model: incident::ActiveModel = incident.clone().into();
    active_model.verified = Set(true);
    active_model.updated_at = Set(chrono::Utc::now().into());
    active_model
        .update(db)
        .await
        .context("Failed to verify incident")?;

    let post = snapshot_incident(
        &IncidentEntity::find_by_id(incident.id)
            .one(db)
            .await?
            .context("Incident not found after update")?,
    );

    log_review_action(
        db,
        ctx,
        incident.id,
        ReviewAction::Accepted,
        justification,
        confidence,
        Some(prior),
        Some(post),
    )
    .await?;

    println!("  ✓ Incident accepted and marked as verified");
    Ok(())
}

/// Reject an incident (delete it)
async fn reject_incident(
    db: &DatabaseConnection,
    ctx: &ReviewContext,
    incident: &IncidentModel,
    justification: Option<String>,
) -> Result<()> {
    let prior = snapshot_incident(incident);

    if ctx.dry_run {
        println!(
            "  [DRY RUN] Would reject (delete) incident {}",
            incident.id
        );
        log_review_action(
            db,
            ctx,
            incident.id,
            ReviewAction::Rejected,
            justification.clone(),
            None,
            Some(prior),
            None,
        )
        .await?;
        return Ok(());
    }

    let active_model: incident::ActiveModel = incident.clone().into();
    active_model
        .delete(db)
        .await
        .context("Failed to delete incident")?;

    log_review_action(
        db,
        ctx,
        incident.id,
        ReviewAction::Rejected,
        justification,
        None,
        Some(prior),
        None,
    )
    .await?;

    println!("  ✗ Incident rejected and deleted");
    Ok(())
}

/// Interactive edit mode for an incident
async fn edit_incident(
    db: &DatabaseConnection,
    ctx: &ReviewContext,
    incident: &IncidentModel,
) -> Result<bool> {
    println!("\n  Editing fields (press Enter to keep current value):");

    let prior = snapshot_incident(incident);
    let mut active_model: incident::ActiveModel = incident.clone().into();
    let mut modified = false;

    // Edit org_name
    if let Some(new_value) = prompt_value("Organization Name", &incident.org_name).await? {
        active_model.org_name = Set(new_value);
        modified = true;
    }

    // Edit org_sector
    if let Some(new_value) = prompt_value("Sector", &incident.org_sector).await? {
        active_model.org_sector = Set(new_value);
        modified = true;
    }

    // Edit attack_type
    if let Some(new_value) = prompt_value("Attack Type", &incident.attack_type).await? {
        active_model.attack_type = Set(new_value);
        modified = true;
    }

    // Edit notes
    match prompt_optional("Notes", &incident.notes).await? {
        Some(new_value) if new_value.is_empty() => {
            active_model.notes = Set(None);
            modified = true;
        }
        Some(new_value) => {
            active_model.notes = Set(Some(new_value));
            modified = true;
        }
        None => {}
    }

    if modified {
        active_model.updated_at = Set(chrono::Utc::now().into());

        if ctx.dry_run {
            println!("  [DRY RUN] Would update incident {}", incident.id);
        } else {
            active_model
                .update(db)
                .await
                .context("Failed to update incident")?;
            println!("  ✓ Incident updated");
        }

        let post = if ctx.dry_run {
            prior.clone()
        } else {
            snapshot_incident(
                &IncidentEntity::find_by_id(incident.id)
                    .one(db)
                    .await?
                    .context("Incident not found after update")?,
            )
        };

        // Log edit action
        let edit_justification = if ctx.dry_run {
            Some("[DRY RUN] Field edits".to_string())
        } else {
            Some("Field edits via interactive review".to_string())
        };

        log_review_action(
            db,
            ctx,
            incident.id,
            ReviewAction::Edited,
            edit_justification,
            None,
            Some(prior),
            Some(post),
        )
        .await?;
    } else {
        println!("  No changes made");
    }

    Ok(modified)
}

/// Run interactive review of unverified incidents
async fn run_interactive(
    db: &DatabaseConnection,
    ctx: &ReviewContext,
    args: &ReviewArgs,
) -> Result<()> {
    let incidents = fetch_unverified(db, args).await?;

    if incidents.is_empty() {
        println!("\nNo unverified incidents found matching the criteria.");
        return Ok(());
    }

    println!(
        "\nFound {} unverified incident(s) to review.\n",
        incidents.len()
    );

    if ctx.dry_run {
        println!("*** DRY RUN MODE — no changes will be committed ***\n");
    }

    let total = incidents.len();
    let mut accepted = 0usize;
    let mut rejected = 0usize;
    let mut skipped = 0usize;
    let mut edited = 0usize;

    for (idx, incident) in incidents.iter().enumerate() {
        display_incident(incident, idx, total);
        display_actions();

        let choice = read_line().await?;

        match choice.as_str() {
            "a" | "accept" => {
                let confidence = prompt_confidence().await?;
                let justification = prompt_justification(false).await?;

                if let Err(e) = accept_incident(
                    db,
                    ctx,
                    incident,
                    justification.clone(),
                    Some(confidence),
                )
                .await
                {
                    warn!("Failed to accept incident {}: {}", incident.id, e);
                    println!("  Error accepting incident: {}", e);
                    skipped += 1;
                } else {
                    accepted += 1;
                }
            }
            "r" | "reject" => {
                let justification = if args.force {
                    Some("Forced rejection without justification".to_string())
                } else {
                    match prompt_justification(true).await {
                        Ok(j) => j,
                        Err(e) => {
                            println!("  {}", e);
                            println!("  Use --force to reject without justification.");
                            skipped += 1;
                            continue;
                        }
                    }
                };

                print!("  Are you sure you want to delete this incident? [y/N]: ");
                let confirm = read_line().await?;
                if confirm == "y" || confirm == "yes" {
                    if let Err(e) = reject_incident(db, ctx, incident, justification.clone()).await
                    {
                        warn!("Failed to reject incident {}: {}", incident.id, e);
                        println!("  Error rejecting incident: {}", e);
                        skipped += 1;
                    } else {
                        rejected += 1;
                    }
                } else {
                    println!("  Cancelled. Skipping...");
                    skipped += 1;
                }
            }
            "e" | "edit" => {
                match edit_incident(db, ctx, incident).await {
                    Ok(did_edit) => {
                        if did_edit {
                            edited += 1;
                        }
                        // After editing, ask if they want to accept
                        print!("\n  Accept this incident? [y/N]: ");
                        let confirm = read_line().await?;
                        if confirm == "y" || confirm == "yes" {
                            let confidence = prompt_confidence().await?;
                            let justification = prompt_justification(false).await?;
                            if let Err(e) = accept_incident(
                                db,
                                ctx,
                                incident,
                                justification.clone(),
                                Some(confidence),
                            )
                            .await
                            {
                                warn!("Failed to accept incident {}: {}", incident.id, e);
                                println!("  Error accepting incident: {}", e);
                            } else {
                                accepted += 1;
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to edit incident {}: {}", incident.id, e);
                        println!("  Error editing incident: {}", e);
                        skipped += 1;
                    }
                }
            }
            "s" | "skip" => {
                println!("  Skipped.");
                skipped += 1;
            }
            "q" | "quit" => {
                println!("\n  Quitting review session.");
                break;
            }
            _ => {
                println!("  Unknown choice. Skipping...");
                skipped += 1;
            }
        }
    }

    println!("\n{}", "═".repeat(70));
    println!("  Review Summary");
    println!("{}", "═".repeat(70));
    println!("  Accepted:  {}", accepted);
    println!("  Rejected:  {}", rejected);
    println!("  Edited:    {}", edited);
    println!("  Skipped:   {}", skipped);
    if ctx.dry_run {
        println!("  *** DRY RUN — no changes committed ***");
    }
    println!("{}", "═".repeat(70));

    Ok(())
}

/// Run batch review mode
async fn run_batch(
    db: &DatabaseConnection,
    ctx: &ReviewContext,
    args: &ReviewArgs,
) -> Result<()> {
    let incidents = fetch_unverified(db, args).await?;

    if incidents.is_empty() {
        println!("\nNo unverified incidents found matching the criteria.");
        return Ok(());
    }

    if args.auto_accept && args.auto_reject {
        bail!("Cannot specify both --auto-accept and --auto-reject");
    }

    if !args.auto_accept && !args.auto_reject {
        println!("\nBatch mode requires --auto-accept or --auto-reject flag.");
        println!("Use interactive mode (without --batch) for manual review.");
        return Ok(());
    }

    let action = if args.auto_accept { "accept" } else { "reject" };
    println!("\nBatch mode: {} {} incident(s)", action, incidents.len());

    if ctx.dry_run {
        println!("*** DRY RUN MODE — no changes will be committed ***\n");
    }

    // Batch mode requires justification
    let justification = if args.justification.is_some() {
        args.justification.clone()
    } else if args.auto_reject && args.force {
        Some("Batch auto-reject (forced)".to_string())
    } else {
        bail!("Batch mode requires --justification. Use --force with --auto-reject to skip.");
    };

    let confidence = args.confidence;

    let mut processed = 0usize;
    let mut failed = 0usize;

    for incident in &incidents {
        let result = if args.auto_accept {
            accept_incident(
                db,
                ctx,
                incident,
                justification.clone(),
                confidence,
            )
            .await
        } else {
            reject_incident(db, ctx, incident, justification.clone()).await
        };

        if let Err(e) = result {
            warn!(
                "Failed to {} incident {}: {}",
                action, incident.id, e
            );
            failed += 1;
        } else {
            processed += 1;
        }
    }

    println!("\n  Processed: {}", processed);
    println!("  Failed:    {}", failed);
    if ctx.dry_run {
        println!("  *** DRY RUN — no changes committed ***");
    }

    Ok(())
}

/// Run audit trail query mode
async fn run_audit_trail_query(db: &DatabaseConnection, incident_id: Uuid) -> Result<()> {
    let incident = IncidentEntity::find_by_id(incident_id)
        .one(db)
        .await?
        .context("Incident not found")?;

    let entries = fetch_audit_trail(db, incident_id).await?;
    display_audit_trail(&incident, &entries);

    Ok(())
}

/// Main entry point for the review command
pub async fn run(args: ReviewArgs) -> Result<()> {
    info!("Starting incident review...");

    let db = get_db().await?;

    // Audit trail query mode takes precedence
    if let Some(incident_id) = args.audit_trail {
        return run_audit_trail_query(&db, incident_id).await;
    }

    let reviewer_id = get_reviewer_id(args.reviewer_id.clone()).await?;
    let ctx = ReviewContext {
        reviewer_id,
        dry_run: args.dry_run,
    };

    if args.batch {
        run_batch(&db, &ctx, &args).await
    } else {
        run_interactive(&db, &ctx, &args).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_incident() -> IncidentModel {
        IncidentModel {
            id: Uuid::new_v4(),
            org_name: "Test Bank".to_string(),
            org_sector: "BANKING".to_string(),
            incident_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap().into(),
            disclosure_date: chrono::NaiveDate::from_ymd_opt(2024, 2, 1).unwrap().into(),
            attack_type: "RANSOMWARE".to_string(),
            data_categories: serde_json::json!(["PII", "Credentials"]),
            record_count_estimate: Some(10000),
            financial_impact_idr: Some(500000000),
            actor_alias: Some("GroupX".to_string()),
            actor_group: Some("APT-1".to_string()),
            source_url: "https://example.com/incident/1".to_string(),
            source_type: "MEDIA".to_string(),
            verified: false,
            notes: Some("Initial report".to_string()),
            created_at: Utc::now().into(),
            updated_at: Utc::now().into(),
        }
    }

    #[test]
    fn test_review_args_default() {
        let args = ReviewArgs::default();
        assert!(!args.batch);
        assert!(!args.auto_accept);
        assert!(!args.auto_reject);
        assert!(args.limit.is_none());
        assert!(args.sector.is_none());
        assert!(args.attack_type.is_none());
        assert!(args.source_type.is_none());
        assert!(args.reviewer_id.is_none());
        assert!(args.justification.is_none());
        assert!(args.confidence.is_none());
        assert!(!args.force);
        assert!(!args.dry_run);
        assert!(args.max_batch_size.is_none());
        assert!(args.audit_trail.is_none());
    }

    #[test]
    fn test_review_args_custom() {
        let args = ReviewArgs {
            batch: true,
            auto_accept: true,
            auto_reject: false,
            limit: Some(10),
            sector: Some("BANKING".to_string()),
            attack_type: Some("RANSOMWARE".to_string()),
            source_type: Some("MEDIA".to_string()),
            reviewer_id: Some("test-reviewer".to_string()),
            justification: Some("Batch acceptance".to_string()),
            confidence: Some(0.75),
            force: false,
            dry_run: true,
            max_batch_size: Some(50),
            audit_trail: None,
        };
        assert!(args.batch);
        assert!(args.auto_accept);
        assert_eq!(args.limit, Some(10));
        assert_eq!(args.sector, Some("BANKING".to_string()));
        assert_eq!(args.reviewer_id, Some("test-reviewer".to_string()));
        assert_eq!(args.confidence, Some(0.75));
        assert!(args.dry_run);
        assert_eq!(args.max_batch_size, Some(50));
    }

    #[tokio::test]
    async fn test_display_incident_output() {
        let incident = create_test_incident();
        // display_incident prints to stdout, so we just verify it doesn't panic
        display_incident(&incident, 0, 1);
    }

    #[tokio::test]
    async fn test_display_actions_output() {
        // display_actions prints to stdout, verify it doesn't panic
        display_actions();
    }

    #[test]
    fn test_snapshot_incident() {
        let incident = create_test_incident();
        let snapshot = snapshot_incident(&incident);
        assert_eq!(snapshot["org_name"], "Test Bank");
        assert_eq!(snapshot["verified"], false);
    }

    #[test]
    fn test_review_action_enum() {
        assert_eq!(ReviewAction::Accepted.as_str(), "ACCEPTED");
        assert_eq!(ReviewAction::Rejected.as_str(), "REJECTED");
        assert_eq!(ReviewAction::Edited.as_str(), "EDITED");
        assert_eq!(ReviewAction::Skipped.as_str(), "SKIPPED");

        assert_eq!(
            "ACCEPTED".parse::<ReviewAction>().unwrap(),
            ReviewAction::Accepted
        );
        assert_eq!(
            "REJECTED".parse::<ReviewAction>().unwrap(),
            ReviewAction::Rejected
        );
        assert!("UNKNOWN".parse::<ReviewAction>().is_err());
    }
}
