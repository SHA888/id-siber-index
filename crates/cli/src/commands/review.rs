//! Interactive review command for unverified incidents
//!
//! This command provides an interactive CLI to review, edit, accept, or reject
//! unverified incident records. It supports both interactive and batch modes.

use anyhow::{Context, Result};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Database, DatabaseConnection, EntityTrait, QueryFilter, Set,
};
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing::{info, warn};
use uuid::Uuid;

use schema::entities::incident::{self, Entity as IncidentEntity, Model as IncidentModel};

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
}

/// Establish a database connection from the DATABASE_URL environment variable
async fn get_db() -> Result<DatabaseConnection> {
    let database_url =
        std::env::var("DATABASE_URL").context("DATABASE_URL environment variable not set")?;
    Database::connect(&database_url)
        .await
        .context("Failed to connect to database")
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

    if let Some(limit) = args.limit {
        results.truncate(limit);
    }

    Ok(results)
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

/// Interactive edit mode for an incident
async fn edit_incident(db: &DatabaseConnection, incident: &IncidentModel) -> Result<bool> {
    println!("\n  Editing fields (press Enter to keep current value):");

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
        active_model
            .update(db)
            .await
            .context("Failed to update incident")?;
        println!("  ✓ Incident updated");
    } else {
        println!("  No changes made");
    }

    Ok(modified)
}

/// Accept an incident (mark as verified)
async fn accept_incident(db: &DatabaseConnection, id: Uuid) -> Result<()> {
    let model = IncidentEntity::find_by_id(id)
        .one(db)
        .await
        .context("Failed to find incident")?
        .context("Incident not found")?;

    let mut active_model: incident::ActiveModel = model.into();
    active_model.verified = Set(true);
    active_model.updated_at = Set(chrono::Utc::now().into());
    active_model
        .update(db)
        .await
        .context("Failed to verify incident")?;

    println!("  ✓ Incident accepted and marked as verified");
    Ok(())
}

/// Reject an incident (delete it)
async fn reject_incident(db: &DatabaseConnection, id: Uuid) -> Result<()> {
    let model = IncidentEntity::find_by_id(id)
        .one(db)
        .await
        .context("Failed to find incident")?
        .context("Incident not found")?;

    let active_model: incident::ActiveModel = model.into();
    active_model
        .delete(db)
        .await
        .context("Failed to delete incident")?;

    println!("  ✗ Incident rejected and deleted");
    Ok(())
}

/// Run interactive review of unverified incidents
async fn run_interactive(db: &DatabaseConnection, args: &ReviewArgs) -> Result<()> {
    let incidents = fetch_unverified(db, args).await?;

    if incidents.is_empty() {
        println!("\nNo unverified incidents found matching the criteria.");
        return Ok(());
    }

    println!(
        "\nFound {} unverified incident(s) to review.\n",
        incidents.len()
    );

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
                if let Err(e) = accept_incident(db, incident.id).await {
                    warn!("Failed to accept incident {}: {}", incident.id, e);
                    println!("  Error accepting incident: {}", e);
                    skipped += 1;
                } else {
                    accepted += 1;
                }
            }
            "r" | "reject" => {
                print!("  Are you sure you want to delete this incident? [y/N]: ");
                let confirm = read_line().await?;
                if confirm == "y" || confirm == "yes" {
                    if let Err(e) = reject_incident(db, incident.id).await {
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
                match edit_incident(db, incident).await {
                    Ok(did_edit) => {
                        if did_edit {
                            edited += 1;
                        }
                        // After editing, ask if they want to accept
                        print!("\n  Accept this incident? [y/N]: ");
                        let confirm = read_line().await?;
                        if confirm == "y" || confirm == "yes" {
                            if let Err(e) = accept_incident(db, incident.id).await {
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
    println!("{}", "═".repeat(70));

    Ok(())
}

/// Run batch review mode
async fn run_batch(db: &DatabaseConnection, args: &ReviewArgs) -> Result<()> {
    let incidents = fetch_unverified(db, args).await?;

    if incidents.is_empty() {
        println!("\nNo unverified incidents found matching the criteria.");
        return Ok(());
    }

    if args.auto_accept && args.auto_reject {
        anyhow::bail!("Cannot specify both --auto-accept and --auto-reject");
    }

    if !args.auto_accept && !args.auto_reject {
        println!("\nBatch mode requires --auto-accept or --auto-reject flag.");
        println!("Use interactive mode (without --batch) for manual review.");
        return Ok(());
    }

    let action = if args.auto_accept { "accept" } else { "reject" };
    println!("\nBatch mode: {} {} incident(s)", action, incidents.len());

    let mut processed = 0usize;
    let mut failed = 0usize;

    for incident in &incidents {
        let result = if args.auto_accept {
            accept_incident(db, incident.id).await
        } else {
            reject_incident(db, incident.id).await
        };

        if let Err(e) = result {
            warn!("Failed to {} incident {}: {}", action, incident.id, e);
            failed += 1;
        } else {
            processed += 1;
        }
    }

    println!("\n  Processed: {}", processed);
    println!("  Failed:    {}", failed);

    Ok(())
}

/// Main entry point for the review command
pub async fn run(args: ReviewArgs) -> Result<()> {
    info!("Starting incident review...");

    let db = get_db().await?;

    if args.batch {
        run_batch(&db, &args).await
    } else {
        run_interactive(&db, &args).await
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
        };
        assert!(args.batch);
        assert!(args.auto_accept);
        assert_eq!(args.limit, Some(10));
        assert_eq!(args.sector, Some("BANKING".to_string()));
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
}
