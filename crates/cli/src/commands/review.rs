//! Verification CLI for reviewing IncidentDraft records
//!
//! This module provides an interactive CLI to review, accept, reject, or edit
//! IncidentDraft records before they are marked as verified incidents.

use anyhow::Result;
use chrono::NaiveDate;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use std::path::PathBuf;

use schema::models::incident::CreateIncident;
use crawler::incident_draft::IncidentDraft;

/// Review command options
#[derive(Debug, Clone)]
pub struct ReviewOptions {
    /// Path to file containing IncidentDraft records
    pub input_file: Option<PathBuf>,
    /// Output file for verified incidents
    pub output_file: Option<PathBuf>,
    /// Batch mode (non-interactive)
    pub batch: bool,
    /// Auto-accept all records (dangerous!)
    pub auto_accept: bool,
    /// Filter by minimum confidence score
    pub min_confidence: Option<f32>,
    /// Limit number of records to review
    pub limit: Option<usize>,
}

/// Review result for a single record
#[derive(Debug, Clone)]
pub enum ReviewDecision {
    Accept(CreateIncident),
    Reject(String), // Reason for rejection
    Edit(CreateIncident),
    Skip,
}

/// Run the review CLI
pub async fn run(options: ReviewOptions) -> Result<()> {
    println!("Starting incident review process...");
    
    // Load incident drafts (for now, we'll simulate loading)
    // TODO: Implement actual loading from database or file
    let drafts = load_incident_drafts(&options).await?;
    
    if drafts.is_empty() {
        println!("No incident drafts found to review.");
        return Ok(());
    }
    
    println!("Found {} incident draft(s) to review.", drafts.len());
    
    let mut accepted = 0;
    let mut rejected = 0;
    let mut edited = 0;
    let mut skipped = 0;
    
    for (index, draft) in drafts.iter().enumerate() {
        if let Some(limit) = options.limit {
            if index >= limit {
                println!("Reached review limit of {}.", limit);
                break;
            }
        }
        
        println!("\n--- Reviewing Draft {} of {} ---", index + 1, drafts.len());
        display_draft_summary(draft);
        
        let decision = if options.batch || options.auto_accept {
            // Batch mode: auto-accept or use simple rules
            if options.auto_accept {
                println!("Auto-accepting draft...");
                ReviewDecision::Accept(convert_draft_to_incident(draft))
            } else {
                batch_review_draft(draft, &options).await?
            }
        } else {
            // Interactive mode
            interactive_review_draft(draft).await?
        };
        
        match decision {
            ReviewDecision::Accept(_incident) => {
                println!("✓ Accepted draft as verified incident.");
                // TODO: Save to database
                accepted += 1;
            }
            ReviewDecision::Reject(reason) => {
                println!("✗ Rejected draft: {}", reason);
                // TODO: Log rejection with reason
                rejected += 1;
            }
            ReviewDecision::Edit(_incident) => {
                println!("✎ Edited and accepted draft.");
                // TODO: Save edited version to database
                edited += 1;
            }
            ReviewDecision::Skip => {
                println!("⏭ Skipped draft.");
                skipped += 1;
            }
        }
    }
    
    println!("\n--- Review Summary ---");
    println!("Accepted: {}", accepted);
    println!("Rejected: {}", rejected);
    println!("Edited: {}", edited);
    println!("Skipped: {}", skipped);
    
    Ok(())
}

/// Load incident drafts from source
async fn load_incident_drafts(_options: &ReviewOptions) -> Result<Vec<IncidentDraft>> {
    // TODO: Implement actual loading from:
    // 1. Database table for unverified incidents
    // 2. JSON/CSV file
    // 3. Crawler output
    
    // For now, return some example drafts
    let example_drafts = vec![
        IncidentDraft::new(
            "PT Bank Contoh Tbk".to_string(),
            NaiveDate::from_ymd_opt(2024, 5, 15).unwrap(),
            "https://example.com/disclosure".to_string(),
            "IDX_DISCLOSURE".to_string(),
        )
        .with_attack_type(Some("RANSOMWARE".to_string()))
        .with_org_sector(Some("BANKING".to_string()))
        .with_data_categories(vec!["PERSONAL_DATA".to_string()])
        .with_confidence(0.8)
        .with_notes(Some("Example ransomware attack on bank".to_string())),
        
        IncidentDraft::new(
            "Rumah Sakit Sehat".to_string(),
            NaiveDate::from_ymd_opt(2024, 5, 10).unwrap(),
            "https://news.com/breach".to_string(),
            "NEWS_ARTICLE".to_string(),
        )
        .with_attack_type(Some("DATA_BREACH".to_string()))
        .with_org_sector(Some("HEALTHCARE".to_string()))
        .with_data_categories(vec!["HEALTH_INFORMATION".to_string(), "PERSONAL_DATA".to_string()])
        .with_confidence(0.6)
        .with_notes(Some("Potential data breach at hospital".to_string())),
    ];
    
    Ok(example_drafts)
}

/// Display a summary of the incident draft
fn display_draft_summary(draft: &IncidentDraft) {
    println!("Organization: {}", draft.org_name);
    if let Some(sector) = &draft.org_sector {
        println!("Sector: {}", sector);
    }
    println!("Disclosure Date: {}", draft.disclosure_date);
    if let Some(incident_date) = draft.incident_date {
        println!("Incident Date: {}", incident_date);
    }
    if let Some(attack_type) = &draft.attack_type {
        println!("Attack Type: {}", attack_type);
    }
    if !draft.data_categories.is_empty() {
        println!("Data Categories: {}", draft.data_categories.join(", "));
    }
    println!("Source: {}", draft.source_url);
    println!("Source Type: {}", draft.source_type);
    println!("Confidence: {:.1}%", draft.confidence * 100.0);
    if let Some(notes) = &draft.notes {
        println!("Notes: {}", notes);
    }
}

/// Interactive review of a single draft
async fn interactive_review_draft(draft: &IncidentDraft) -> Result<ReviewDecision> {
    let theme = ColorfulTheme::default();
    
    let choices = vec![
        "Accept (mark as verified)",
        "Reject (discard draft)",
        "Edit fields",
        "Skip (review later)",
        "View full details",
    ];
    
    loop {
        let selection = Select::with_theme(&theme)
            .with_prompt("What would you like to do with this draft?")
            .items(&choices)
            .default(0)
            .interact()?;
        
        match selection {
            0 => {
                // Accept
                if Confirm::with_theme(&theme)
                    .with_prompt("Are you sure you want to accept this draft as a verified incident?")
                    .default(true)
                    .interact()?
                {
                    return Ok(ReviewDecision::Accept(convert_draft_to_incident(draft)));
                }
            }
            1 => {
                // Reject
                let reason = Input::with_theme(&theme)
                    .with_prompt("Reason for rejection")
                    .default("Insufficient evidence".to_string())
                    .interact()?;
                
                if Confirm::with_theme(&theme)
                    .with_prompt(&format!("Reject draft with reason: '{}'?", reason))
                    .default(true)
                    .interact()?
                {
                    return Ok(ReviewDecision::Reject(reason));
                }
            }
            2 => {
                // Edit
                let edited = edit_draft_interactively(draft).await?;
                return Ok(ReviewDecision::Edit(edited));
            }
            3 => {
                // Skip
                return Ok(ReviewDecision::Skip);
            }
            4 => {
                // View full details
                println!("\n--- Full Draft Details ---");
                println!("{:#?}", draft);
                println!("--- End Details ---\n");
                continue;
            }
            _ => unreachable!(),
        }
    }
}

/// Batch review (non-interactive)
async fn batch_review_draft(draft: &IncidentDraft, options: &ReviewOptions) -> Result<ReviewDecision> {
    // Apply review rules
    if let Some(min_confidence) = options.min_confidence {
        if draft.confidence < min_confidence {
            return Ok(ReviewDecision::Reject(format!(
                "Confidence score {:.1}% below minimum {:.1}%",
                draft.confidence * 100.0,
                min_confidence * 100.0
            )));
        }
    }
    
    // Check for required fields
    if draft.org_name.trim().is_empty() {
        return Ok(ReviewDecision::Reject("Missing organization name".to_string()));
    }
    
    if draft.source_url.trim().is_empty() {
        return Ok(ReviewDecision::Reject("Missing source URL".to_string()));
    }
    
    // Auto-accept high-confidence drafts
    if draft.confidence >= 0.9 {
        println!("Auto-accepting high-confidence draft ({:.1}%)", draft.confidence * 100.0);
        return Ok(ReviewDecision::Accept(convert_draft_to_incident(draft)));
    }
    
    // Default to accept for batch mode
    Ok(ReviewDecision::Accept(convert_draft_to_incident(draft)))
}

/// Interactive editing of draft fields
async fn edit_draft_interactively(draft: &IncidentDraft) -> Result<CreateIncident> {
    let theme = ColorfulTheme::default();
    
    println!("Editing draft fields. Press Enter to keep current value.");
    
    let org_name: String = Input::with_theme(&theme)
        .with_prompt("Organization Name")
        .default(draft.org_name.clone())
        .interact()?;
    
    let org_sector: String = Input::with_theme(&theme)
        .with_prompt("Organization Sector")
        .default(draft.org_sector.clone().unwrap_or_else(|| "UNKNOWN".to_string()))
        .interact()?;
    
    let attack_type: String = Input::with_theme(&theme)
        .with_prompt("Attack Type")
        .default(draft.attack_type.clone().unwrap_or_else(|| "UNKNOWN".to_string()))
        .interact()?;
    
    let notes: String = Input::with_theme(&theme)
        .with_prompt("Notes")
        .default(draft.notes.clone().unwrap_or_default())
        .interact()?;
    
    // Convert to CreateIncident
    let incident = CreateIncident {
        org_name,
        org_sector,
        incident_date: draft.incident_date.unwrap_or(draft.disclosure_date).into(),
        disclosure_date: draft.disclosure_date.into(),
        attack_type,
        data_categories: draft.data_categories.clone(),
        record_count_estimate: draft.record_count_estimate,
        financial_impact_idr: draft.financial_impact_idr,
        actor_alias: draft.actor_alias.clone(),
        actor_group: draft.actor_group.clone(),
        source_url: draft.source_url.clone(),
        source_type: draft.source_type.clone(),
        notes: if notes.is_empty() { None } else { Some(notes) },
    };
    
    Ok(incident)
}

/// Convert IncidentDraft to CreateIncident for database insertion
fn convert_draft_to_incident(draft: &IncidentDraft) -> CreateIncident {
    CreateIncident {
        org_name: draft.org_name.clone(),
        org_sector: draft.org_sector.clone().unwrap_or_else(|| "UNKNOWN".to_string()),
        incident_date: draft.incident_date.unwrap_or(draft.disclosure_date).into(),
        disclosure_date: draft.disclosure_date.into(),
        attack_type: draft.attack_type.clone().unwrap_or_else(|| "UNKNOWN".to_string()),
        data_categories: draft.data_categories.clone(),
        record_count_estimate: draft.record_count_estimate,
        financial_impact_idr: draft.financial_impact_idr,
        actor_alias: draft.actor_alias.clone(),
        actor_group: draft.actor_group.clone(),
        source_url: draft.source_url.clone(),
        source_type: draft.source_type.clone(),
        notes: draft.notes.clone(),
    }
}

/// Batch review mode for historical backfill
pub async fn batch_review_all(options: ReviewOptions) -> Result<()> {
    println!("Starting batch review of all incident drafts...");
    
    let review_options = ReviewOptions {
        batch: true,
        ..options
    };
    
    run(review_options).await
}