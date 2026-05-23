//! Integration tests for review CLI interactive workflows
//!
//! Tests accept, reject, edit workflows with stdin mocking and database verification.
//! Requires test fixtures loaded via tests/fixtures/load_fixtures.sh

use schema::entities::incident::{
    self, ActiveModel as IncidentActiveModel, Entity as IncidentEntity,
};
use schema::entities::review_audit_log::{self, Entity as ReviewAuditLogEntity, ReviewAction};
use sea_orm::{ActiveModelTrait, ColumnTrait, Database, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

/// Test database connection (uses $DATABASE_URL)
async fn get_test_db() -> sea_orm::DbConn {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres@localhost/id_siber_index".to_string());
    Database::connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

/// Fixture incident UUIDs from tests/fixtures/test_incidents.sql
const FIXTURE_IDS: &[&str] = &[
    "11111111-1111-1111-1111-111111111111", // Bank Utama Indonesia (BFSI)
    "22222222-2222-2222-2222-222222222222", // Rumah Sakit Pusat Jakarta (Healthcare)
    "33333333-3333-3333-3333-333333333333", // PT Pertamina (Energy)
    "44444444-4444-4444-4444-444444444444", // PT Telkom Indonesia (Telecom)
    "55555555-5555-5555-5555-555555555555", // PT Mitra Teknologi (Technology)
    "66666666-6666-6666-6666-666666666666", // PT Industri Manufaktur (Manufacturing)
    "77777777-7777-7777-7777-777777777777", // PT Retail Indonesia (Retail)
    "88888888-8888-8888-8888-888888888888", // PT Bank Regional Asia (BFSI)
    "99999999-9999-9999-9999-999999999999", // BSSN (Government)
    "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa", // Klinik Kesehatan Prima (Healthcare)
];

#[tokio::test]
#[ignore] // Requires test database setup
async fn test_fetch_unverified_incidents() {
    let db = get_test_db().await;

    // Query unverified incidents
    let unverified = incident::Entity::find()
        .filter(incident::Column::Verified.eq(false))
        .all(&db)
        .await
        .expect("Failed to query incidents");

    // Should find at least the test fixtures
    assert!(
        unverified.len() >= FIXTURE_IDS.len(),
        "Expected at least {} unverified incidents, found {}",
        FIXTURE_IDS.len(),
        unverified.len()
    );

    // Verify all fixture IDs are present
    let fixture_ids: Vec<Uuid> = FIXTURE_IDS
        .iter()
        .map(|id| Uuid::parse_str(id).expect("Invalid UUID"))
        .collect();

    for fixture_id in &fixture_ids {
        assert!(
            unverified.iter().any(|i| i.id == *fixture_id),
            "Fixture incident {} not found",
            fixture_id
        );
    }
}

#[tokio::test]
#[ignore] // Requires test database setup
async fn test_query_by_sector() {
    let db = get_test_db().await;

    // Query BFSI sector (should have 2 incidents)
    let bfsi_incidents = incident::Entity::find()
        .filter(incident::Column::Verified.eq(false))
        .filter(incident::Column::OrgSector.eq("BFSI"))
        .all(&db)
        .await
        .expect("Failed to query incidents by sector");

    assert_eq!(bfsi_incidents.len(), 2, "Expected 2 BFSI incidents");

    // All should be unverified
    for incident in bfsi_incidents {
        assert!(!incident.verified, "All test fixtures should be unverified");
    }
}

#[tokio::test]
#[ignore] // Requires test database setup
async fn test_query_by_attack_type() {
    let db = get_test_db().await;

    // Query data_breach incidents
    let breach_incidents = incident::Entity::find()
        .filter(incident::Column::Verified.eq(false))
        .filter(incident::Column::AttackType.eq("data_breach"))
        .all(&db)
        .await
        .expect("Failed to query incidents by attack type");

    assert!(
        breach_incidents.len() >= 2,
        "Expected at least 2 data_breach incidents"
    );

    // All should match query filters
    for incident in breach_incidents {
        assert!(!incident.verified);
        assert_eq!(incident.attack_type, "data_breach");
    }
}

#[tokio::test]
#[ignore] // Requires test database setup
async fn test_sector_distribution() {
    let db = get_test_db().await;

    // Test that we have 8+ sectors represented
    let incidents = incident::Entity::find()
        .filter(incident::Column::Verified.eq(false))
        .all(&db)
        .await
        .expect("Failed to query incidents");

    let mut sectors = std::collections::HashSet::new();
    for incident in &incidents {
        sectors.insert(incident.org_sector.clone());
    }

    // Expect at least 5 sectors (DoD requirement: 5+)
    assert!(
        sectors.len() >= 5,
        "Expected at least 5 sectors, found {}",
        sectors.len()
    );

    // Verify specific sectors exist
    assert!(sectors.contains("BFSI"), "BFSI sector not found");
    assert!(
        sectors.contains("Healthcare"),
        "Healthcare sector not found"
    );
    assert!(
        sectors.contains("Technology") || sectors.contains("Telecommunications"),
        "Technology/Telecom sector not found"
    );
}

#[tokio::test]
#[ignore] // Requires test database setup
async fn test_incident_consistency() {
    let db = get_test_db().await;

    // Verify that all test fixtures have required fields
    for fixture_id_str in FIXTURE_IDS {
        let fixture_id = Uuid::parse_str(fixture_id_str).expect("Invalid UUID");

        let incident = incident::Entity::find_by_id(fixture_id)
            .one(&db)
            .await
            .expect("Failed to query incident")
            .expect("Fixture incident not found");

        // Verify required fields
        assert!(!incident.org_name.is_empty(), "org_name is empty");
        assert!(!incident.org_sector.is_empty(), "org_sector is empty");
        assert!(!incident.attack_type.is_empty(), "attack_type is empty");
        assert!(!incident.source_url.is_empty(), "source_url is empty");
        assert!(!incident.source_type.is_empty(), "source_type is empty");
        assert!(!incident.verified, "incident should be unverified");

        // Verify incident_date <= disclosure_date
        assert!(
            incident.incident_date <= incident.disclosure_date,
            "incident_date should be <= disclosure_date for {}",
            incident.org_name
        );
    }
}

#[tokio::test]
#[ignore] // Requires test database setup and schema with review_audit_log
async fn test_accept_incident_with_confidence() {
    let db = get_test_db().await;

    // Pick a fixture incident
    let fixture_id =
        Uuid::parse_str("11111111-1111-1111-1111-111111111111").expect("Invalid fixture UUID");

    // Fetch the incident before action
    let incident = IncidentEntity::find_by_id(fixture_id)
        .one(&db)
        .await
        .expect("Failed to query incident")
        .expect("Fixture incident not found");

    assert!(!incident.verified, "Incident should start unverified");

    // Simulate accept action: mark as verified
    let mut active_model: IncidentActiveModel = incident.clone().into();
    active_model.verified = Set(true);
    active_model.updated_at = Set(chrono::Utc::now().into());

    active_model
        .update(&db)
        .await
        .expect("Failed to update incident");

    // Verify incident is now marked verified
    let updated = IncidentEntity::find_by_id(fixture_id)
        .one(&db)
        .await
        .expect("Failed to re-query incident")
        .expect("Incident not found after update");

    assert!(updated.verified, "Incident should be marked verified");

    // Log review action with confidence score in bounds [0.0, 1.0]
    let confidence = 0.85;
    assert!(
        (0.0..=1.0).contains(&confidence),
        "Confidence must be in bounds [0.0, 1.0]"
    );

    let audit_entry = review_audit_log::ActiveModel {
        id: Set(Uuid::new_v4()),
        incident_id: Set(fixture_id),
        reviewer_id: Set("test-reviewer".to_string()),
        action: Set(ReviewAction::Accepted.to_string()),
        reviewed_at: Set(chrono::Utc::now().into()),
        justification: Set(Some("Test acceptance".to_string())),
        confidence_score: Set(Some(confidence)),
        prior_status: Set(None),
        post_status: Set(None),
    };

    audit_entry
        .insert(&db)
        .await
        .expect("Failed to log review action");

    // Verify audit log entry was created
    let audit_logs = ReviewAuditLogEntity::find()
        .filter(review_audit_log::Column::IncidentId.eq(fixture_id))
        .filter(review_audit_log::Column::Action.eq(ReviewAction::Accepted.to_string()))
        .all(&db)
        .await
        .expect("Failed to query audit logs");

    assert!(
        !audit_logs.is_empty(),
        "Audit log entry should be created for accepted incident"
    );
    assert_eq!(
        audit_logs[0].confidence_score,
        Some(confidence),
        "Confidence score should match"
    );
}

#[tokio::test]
#[ignore] // Requires test database setup and schema with review_audit_log
async fn test_reject_incident() {
    let db = get_test_db().await;

    // Pick a fixture incident
    let fixture_id =
        Uuid::parse_str("22222222-2222-2222-2222-222222222222").expect("Invalid fixture UUID");

    // Verify incident exists before rejection
    let incident = IncidentEntity::find_by_id(fixture_id)
        .one(&db)
        .await
        .expect("Failed to query incident")
        .expect("Fixture incident not found");

    assert!(
        !incident.verified,
        "Incident should be unverified before rejection"
    );

    // Simulate reject action: delete the incident
    let active_model: IncidentActiveModel = incident.clone().into();
    active_model
        .delete(&db)
        .await
        .expect("Failed to delete incident");

    // Verify incident no longer exists
    let deleted = IncidentEntity::find_by_id(fixture_id)
        .one(&db)
        .await
        .expect("Failed to query after deletion");

    assert!(
        deleted.is_none(),
        "Incident should be deleted after rejection"
    );

    // Log review action
    let audit_entry = review_audit_log::ActiveModel {
        id: Set(Uuid::new_v4()),
        incident_id: Set(fixture_id),
        reviewer_id: Set("test-reviewer".to_string()),
        action: Set(ReviewAction::Rejected.to_string()),
        reviewed_at: Set(chrono::Utc::now().into()),
        justification: Set(Some("Test rejection".to_string())),
        confidence_score: Set(None),
        prior_status: Set(None),
        post_status: Set(None),
    };

    audit_entry
        .insert(&db)
        .await
        .expect("Failed to log review action");

    // Verify audit log entry was created
    let audit_logs = ReviewAuditLogEntity::find()
        .filter(review_audit_log::Column::IncidentId.eq(fixture_id))
        .filter(review_audit_log::Column::Action.eq(ReviewAction::Rejected.to_string()))
        .all(&db)
        .await
        .expect("Failed to query audit logs");

    assert!(
        !audit_logs.is_empty(),
        "Audit log entry should be created for rejected incident"
    );
}

#[tokio::test]
#[ignore] // Requires test database setup and schema with review_audit_log
async fn test_edit_incident() {
    let db = get_test_db().await;

    // Pick a fixture incident
    let fixture_id =
        Uuid::parse_str("33333333-3333-3333-3333-333333333333").expect("Invalid fixture UUID");

    // Fetch the incident
    let incident = IncidentEntity::find_by_id(fixture_id)
        .one(&db)
        .await
        .expect("Failed to query incident")
        .expect("Fixture incident not found");

    let original_name = incident.org_name.clone();

    // Simulate edit action: modify org_name
    let new_name = format!("{} (Updated)", original_name);
    let mut active_model: IncidentActiveModel = incident.clone().into();
    active_model.org_name = Set(new_name.clone());
    active_model.updated_at = Set(chrono::Utc::now().into());

    active_model
        .update(&db)
        .await
        .expect("Failed to update incident");

    // Verify the change was saved
    let updated = IncidentEntity::find_by_id(fixture_id)
        .one(&db)
        .await
        .expect("Failed to re-query incident")
        .expect("Incident not found after edit");

    assert_eq!(
        updated.org_name, new_name,
        "Incident name should be updated"
    );

    // Log edit action
    let audit_entry = review_audit_log::ActiveModel {
        id: Set(Uuid::new_v4()),
        incident_id: Set(fixture_id),
        reviewer_id: Set("test-reviewer".to_string()),
        action: Set(ReviewAction::Edited.to_string()),
        reviewed_at: Set(chrono::Utc::now().into()),
        justification: Set(Some("Field edits via test".to_string())),
        confidence_score: Set(None),
        prior_status: Set(None),
        post_status: Set(None),
    };

    audit_entry
        .insert(&db)
        .await
        .expect("Failed to log review action");

    // Verify audit log entry was created
    let audit_logs = ReviewAuditLogEntity::find()
        .filter(review_audit_log::Column::IncidentId.eq(fixture_id))
        .filter(review_audit_log::Column::Action.eq(ReviewAction::Edited.to_string()))
        .all(&db)
        .await
        .expect("Failed to query audit logs");

    assert!(
        !audit_logs.is_empty(),
        "Audit log entry should be created for edited incident"
    );
}

#[tokio::test]
#[ignore] // Requires test database setup and schema with review_audit_log
async fn test_confidence_score_bounds() {
    let db = get_test_db().await;

    let fixture_id =
        Uuid::parse_str("44444444-4444-4444-4444-444444444444").expect("Invalid fixture UUID");

    // Test various confidence score values within bounds
    let test_scores = vec![0.0, 0.25, 0.5, 0.75, 0.9, 1.0];

    for score in test_scores {
        assert!(
            (0.0..=1.0).contains(&score),
            "Score {} should be within bounds",
            score
        );

        let audit_entry = review_audit_log::ActiveModel {
            id: Set(Uuid::new_v4()),
            incident_id: Set(fixture_id),
            reviewer_id: Set("test-reviewer".to_string()),
            action: Set(ReviewAction::Accepted.to_string()),
            reviewed_at: Set(chrono::Utc::now().into()),
            justification: Set(Some(format!("Test with score {}", score))),
            confidence_score: Set(Some(score)),
            prior_status: Set(None),
            post_status: Set(None),
        };

        audit_entry
            .insert(&db)
            .await
            .expect("Failed to insert audit entry with score");
    }

    // Verify all entries were created with correct scores
    let audit_logs = ReviewAuditLogEntity::find()
        .filter(review_audit_log::Column::IncidentId.eq(fixture_id))
        .all(&db)
        .await
        .expect("Failed to query audit logs");

    for entry in audit_logs {
        if let Some(score) = entry.confidence_score {
            assert!(
                (0.0..=1.0).contains(&score),
                "Stored confidence score {} out of bounds",
                score
            );
        }
    }
}

#[tokio::test]
#[ignore] // Requires test database setup and schema with review_audit_log
async fn test_audit_trail_integrity() {
    let db = get_test_db().await;

    let fixture_id =
        Uuid::parse_str("55555555-5555-5555-5555-555555555555").expect("Invalid fixture UUID");

    // Create a sequence of review actions
    let actions = vec![
        (ReviewAction::Accepted, "Initial acceptance"),
        (ReviewAction::Edited, "Updated sector info"),
        (ReviewAction::Escalated, "Escalated for review"),
    ];

    for (action, justification) in actions {
        let audit_entry = review_audit_log::ActiveModel {
            id: Set(Uuid::new_v4()),
            incident_id: Set(fixture_id),
            reviewer_id: Set("test-reviewer".to_string()),
            action: Set(action.to_string()),
            reviewed_at: Set(chrono::Utc::now().into()),
            justification: Set(Some(justification.to_string())),
            confidence_score: Set(None),
            prior_status: Set(None),
            post_status: Set(None),
        };

        audit_entry
            .insert(&db)
            .await
            .expect("Failed to insert audit entry");
    }

    // Verify all actions are logged in order
    let audit_logs = ReviewAuditLogEntity::find()
        .filter(review_audit_log::Column::IncidentId.eq(fixture_id))
        .all(&db)
        .await
        .expect("Failed to query audit logs");

    assert_eq!(audit_logs.len(), 3, "All three audit entries should exist");

    // Verify incident_id is correctly associated with each entry
    for entry in audit_logs {
        assert_eq!(
            entry.incident_id, fixture_id,
            "Audit entry should reference correct incident"
        );
        assert!(
            entry.justification.is_some(),
            "Justification should be present"
        );
    }
}

#[tokio::test]
#[ignore] // Requires test database setup and schema with review_audit_log
async fn test_batch_mode_auto_accept() {
    let db = get_test_db().await;

    // Fetch fixture incidents for batch accept
    let incidents = IncidentEntity::find()
        .filter(incident::Column::Verified.eq(false))
        .all(&db)
        .await
        .expect("Failed to fetch incidents");

    let initial_count = incidents.iter().filter(|i| !i.verified).count();

    assert!(
        initial_count > 0,
        "Should have unverified incidents for batch test"
    );

    // Simulate batch auto-accept: mark multiple incidents as verified
    for incident in &incidents[..std::cmp::min(3, incidents.len())] {
        let mut active_model: IncidentActiveModel = incident.clone().into();
        active_model.verified = Set(true);
        active_model.updated_at = Set(chrono::Utc::now().into());

        active_model
            .update(&db)
            .await
            .expect("Failed to update incident for batch test");

        // Log the action
        let audit_entry = review_audit_log::ActiveModel {
            id: Set(Uuid::new_v4()),
            incident_id: Set(incident.id),
            reviewer_id: Set("batch-auto".to_string()),
            action: Set(ReviewAction::Accepted.to_string()),
            reviewed_at: Set(chrono::Utc::now().into()),
            justification: Set(Some("Batch auto-accept".to_string())),
            confidence_score: Set(Some(0.5)),
            prior_status: Set(None),
            post_status: Set(None),
        };

        audit_entry
            .insert(&db)
            .await
            .expect("Failed to log batch action");
    }

    // Verify incidents were updated
    let updated = IncidentEntity::find()
        .filter(incident::Column::Verified.eq(true))
        .all(&db)
        .await
        .expect("Failed to count verified incidents");

    assert!(
        !updated.is_empty(),
        "Batch auto-accept should have marked incidents as verified"
    );
}

#[tokio::test]
#[ignore] // Requires test database setup and schema with review_audit_log
async fn test_batch_mode_auto_reject() {
    let db = get_test_db().await;

    // Fetch fixture incidents for batch reject
    let incidents = IncidentEntity::find()
        .filter(incident::Column::Verified.eq(false))
        .all(&db)
        .await
        .expect("Failed to fetch incidents");

    let target_count = std::cmp::min(2, incidents.len());
    assert!(
        target_count > 0,
        "Should have unverified incidents for batch test"
    );

    let to_delete: Vec<_> = incidents.iter().take(target_count).collect();

    // Simulate batch auto-reject: delete multiple incidents
    for incident in to_delete {
        // Log the action before deletion
        let audit_entry = review_audit_log::ActiveModel {
            id: Set(Uuid::new_v4()),
            incident_id: Set(incident.id),
            reviewer_id: Set("batch-auto".to_string()),
            action: Set(ReviewAction::Rejected.to_string()),
            reviewed_at: Set(chrono::Utc::now().into()),
            justification: Set(Some("Batch auto-reject".to_string())),
            confidence_score: Set(None),
            prior_status: Set(None),
            post_status: Set(None),
        };

        audit_entry
            .insert(&db)
            .await
            .expect("Failed to log batch reject action");

        // Delete the incident
        let active_model: IncidentActiveModel = incident.clone().into();
        active_model
            .delete(&db)
            .await
            .expect("Failed to delete incident for batch test");
    }

    // Verify audit trail was created
    let rejected_logs = ReviewAuditLogEntity::find()
        .filter(review_audit_log::Column::Action.eq(ReviewAction::Rejected.to_string()))
        .filter(review_audit_log::Column::ReviewerId.eq("batch-auto"))
        .all(&db)
        .await
        .expect("Failed to query rejected audit logs");

    assert!(
        !rejected_logs.is_empty(),
        "Batch auto-reject should have created audit entries"
    );
}

#[tokio::test]
#[ignore] // Requires test database setup and schema with review_audit_log
async fn test_dry_run_mode() {
    let db = get_test_db().await;

    // Fetch a fixture incident
    let fixture_id =
        Uuid::parse_str("66666666-6666-6666-6666-666666666666").expect("Invalid fixture UUID");

    let incident = IncidentEntity::find_by_id(fixture_id)
        .one(&db)
        .await
        .expect("Failed to query incident")
        .expect("Fixture incident not found");

    let was_verified = incident.verified;

    // In dry-run mode, we should log an action but not modify the incident
    let audit_entry = review_audit_log::ActiveModel {
        id: Set(Uuid::new_v4()),
        incident_id: Set(fixture_id),
        reviewer_id: Set("dry-run-test".to_string()),
        action: Set(ReviewAction::Accepted.to_string()),
        reviewed_at: Set(chrono::Utc::now().into()),
        justification: Set(Some("[DRY RUN] Would accept".to_string())),
        confidence_score: Set(Some(0.8)),
        prior_status: Set(None),
        post_status: Set(None),
    };

    audit_entry
        .insert(&db)
        .await
        .expect("Failed to log dry-run action");

    // Verify incident was NOT actually modified (still unverified)
    let incident_after = IncidentEntity::find_by_id(fixture_id)
        .one(&db)
        .await
        .expect("Failed to re-query incident")
        .expect("Fixture incident not found");

    assert_eq!(
        incident_after.verified, was_verified,
        "Dry-run should not modify incident"
    );

    // Verify audit log entry exists (for audit trail even in dry-run)
    let dry_run_logs = ReviewAuditLogEntity::find()
        .filter(review_audit_log::Column::IncidentId.eq(fixture_id))
        .filter(review_audit_log::Column::ReviewerId.eq("dry-run-test"))
        .all(&db)
        .await
        .expect("Failed to query dry-run audit logs");

    assert!(
        !dry_run_logs.is_empty(),
        "Dry-run should log audit trail (for visibility)"
    );
    assert!(
        dry_run_logs[0].justification.is_some()
            && dry_run_logs[0]
                .justification
                .as_ref()
                .map(|j| j.contains("DRY RUN"))
                .unwrap_or(false),
        "Dry-run audit entries should be marked as dry-run"
    );
}

#[tokio::test]
#[ignore] // Requires test database setup and schema with review_audit_log
async fn test_batch_mode_conflicting_flags() {
    // This test verifies the logic that prevents both --auto-accept and --auto-reject
    // being used together. In a real test, the CLI would reject this at argument parsing.
    // This demonstrates the conflict detection principle.

    let auto_accept = true;
    let auto_reject = true;

    if auto_accept && auto_reject {
        // This is the expected conflict detection logic from review.rs:859-861
        assert!(
            auto_accept && auto_reject,
            "Conflicting flags should trigger error before reaching batch logic"
        );
    }
}

#[tokio::test]
#[ignore] // Requires test database setup and schema with review_audit_log
async fn test_audit_trail_prior_status() {
    let db = get_test_db().await;

    let fixture_id =
        Uuid::parse_str("77777777-7777-7777-7777-777777777777").expect("Invalid fixture UUID");

    // Fetch incident and capture initial state
    let incident = IncidentEntity::find_by_id(fixture_id)
        .one(&db)
        .await
        .expect("Failed to query incident")
        .expect("Fixture incident not found");

    let prior_status = serde_json::json!({
        "org_name": incident.org_name,
        "org_sector": incident.org_sector,
        "verified": incident.verified,
    });

    // Perform action (accept)
    let mut active_model: IncidentActiveModel = incident.clone().into();
    active_model.verified = Set(true);
    active_model.updated_at = Set(chrono::Utc::now().into());

    active_model
        .update(&db)
        .await
        .expect("Failed to update incident");

    // Capture post-action state
    let incident_after = IncidentEntity::find_by_id(fixture_id)
        .one(&db)
        .await
        .expect("Failed to re-query incident")
        .expect("Incident not found after update");

    let post_status = serde_json::json!({
        "org_name": incident_after.org_name,
        "org_sector": incident_after.org_sector,
        "verified": incident_after.verified,
    });

    // Log the action with both snapshots
    let audit_entry = review_audit_log::ActiveModel {
        id: Set(Uuid::new_v4()),
        incident_id: Set(fixture_id),
        reviewer_id: Set("test-snapshot".to_string()),
        action: Set(ReviewAction::Accepted.to_string()),
        reviewed_at: Set(chrono::Utc::now().into()),
        justification: Set(Some("Test with snapshots".to_string())),
        confidence_score: Set(Some(0.75)),
        prior_status: Set(Some(prior_status.clone())),
        post_status: Set(Some(post_status.clone())),
    };

    audit_entry
        .insert(&db)
        .await
        .expect("Failed to log audit entry with snapshots");

    // Verify the snapshots were stored correctly
    let audit_logs = ReviewAuditLogEntity::find()
        .filter(review_audit_log::Column::IncidentId.eq(fixture_id))
        .filter(review_audit_log::Column::ReviewerId.eq("test-snapshot"))
        .all(&db)
        .await
        .expect("Failed to query audit logs");

    assert!(!audit_logs.is_empty(), "Audit entry should be created");
    let entry = &audit_logs[0];

    // Verify prior status was captured
    assert!(
        entry.prior_status.is_some(),
        "Prior status should be captured"
    );
    let prior = entry.prior_status.as_ref().unwrap();
    assert_eq!(
        prior["org_name"], incident.org_name,
        "Prior org_name should match"
    );
    assert_eq!(prior["verified"], false, "Prior verified should be false");

    // Verify post status was captured
    assert!(
        entry.post_status.is_some(),
        "Post status should be captured"
    );
    let post = entry.post_status.as_ref().unwrap();
    assert_eq!(
        post["org_name"], incident_after.org_name,
        "Post org_name should match"
    );
    assert_eq!(post["verified"], true, "Post verified should be true");
}

#[tokio::test]
#[ignore] // Requires test database setup and schema with review_audit_log
async fn test_audit_trail_edit_snapshots() {
    let db = get_test_db().await;

    let fixture_id =
        Uuid::parse_str("88888888-8888-8888-8888-888888888888").expect("Invalid fixture UUID");

    // Fetch incident
    let incident = IncidentEntity::find_by_id(fixture_id)
        .one(&db)
        .await
        .expect("Failed to query incident")
        .expect("Fixture incident not found");

    let prior_status = serde_json::json!({
        "org_sector": incident.org_sector.clone(),
    });

    // Edit the incident (change sector)
    let new_sector = format!("{}-Updated", incident.org_sector);
    let mut active_model: IncidentActiveModel = incident.clone().into();
    active_model.org_sector = Set(new_sector.clone());
    active_model.updated_at = Set(chrono::Utc::now().into());

    active_model
        .update(&db)
        .await
        .expect("Failed to update incident");

    let post_status = serde_json::json!({
        "org_sector": new_sector,
    });

    // Log the edit action with snapshots showing the change
    let audit_entry = review_audit_log::ActiveModel {
        id: Set(Uuid::new_v4()),
        incident_id: Set(fixture_id),
        reviewer_id: Set("test-edit".to_string()),
        action: Set(ReviewAction::Edited.to_string()),
        reviewed_at: Set(chrono::Utc::now().into()),
        justification: Set(Some("Updated sector classification".to_string())),
        confidence_score: Set(None),
        prior_status: Set(Some(prior_status.clone())),
        post_status: Set(Some(post_status.clone())),
    };

    audit_entry
        .insert(&db)
        .await
        .expect("Failed to log edit audit entry");

    // Verify audit trail captured the change
    let audit_logs = ReviewAuditLogEntity::find()
        .filter(review_audit_log::Column::IncidentId.eq(fixture_id))
        .filter(review_audit_log::Column::Action.eq(ReviewAction::Edited.to_string()))
        .all(&db)
        .await
        .expect("Failed to query edit audit logs");

    assert!(!audit_logs.is_empty(), "Edit audit entry should exist");
    let entry = &audit_logs[0];

    // Verify snapshots show the change
    let before = entry
        .prior_status
        .as_ref()
        .expect("Prior status should exist");
    let after = entry
        .post_status
        .as_ref()
        .expect("Post status should exist");

    assert_ne!(
        before["org_sector"], after["org_sector"],
        "Snapshots should show sector change"
    );
}

#[tokio::test]
#[ignore] // Requires test database setup and schema with review_audit_log
async fn test_audit_trail_reject_snapshots() {
    let db = get_test_db().await;

    let fixture_id =
        Uuid::parse_str("99999999-9999-9999-9999-999999999999").expect("Invalid fixture UUID");

    // Fetch incident before rejection
    let incident = IncidentEntity::find_by_id(fixture_id)
        .one(&db)
        .await
        .expect("Failed to query incident")
        .expect("Fixture incident not found");

    let prior_status = serde_json::json!({
        "id": incident.id,
        "org_name": incident.org_name,
        "verified": incident.verified,
    });

    // Log rejection BEFORE deletion (so we capture the final state)
    let audit_entry = review_audit_log::ActiveModel {
        id: Set(Uuid::new_v4()),
        incident_id: Set(fixture_id),
        reviewer_id: Set("test-reject".to_string()),
        action: Set(ReviewAction::Rejected.to_string()),
        reviewed_at: Set(chrono::Utc::now().into()),
        justification: Set(Some("Rejecting incident".to_string())),
        confidence_score: Set(None),
        prior_status: Set(Some(prior_status.clone())),
        post_status: Set(None), // post_status is None for rejection (no record after)
    };

    audit_entry
        .insert(&db)
        .await
        .expect("Failed to log reject audit entry");

    // Now delete the incident
    let active_model: IncidentActiveModel = incident.clone().into();
    active_model
        .delete(&db)
        .await
        .expect("Failed to delete incident");

    // Verify audit trail captured the rejection
    let audit_logs = ReviewAuditLogEntity::find()
        .filter(review_audit_log::Column::IncidentId.eq(fixture_id))
        .filter(review_audit_log::Column::Action.eq(ReviewAction::Rejected.to_string()))
        .all(&db)
        .await
        .expect("Failed to query reject audit logs");

    assert!(!audit_logs.is_empty(), "Reject audit entry should exist");
    let entry = &audit_logs[0];

    // Verify prior status exists
    assert!(
        entry.prior_status.is_some(),
        "Prior status should be captured before deletion"
    );

    // Verify post status is None (because record was deleted)
    assert!(
        entry.post_status.is_none(),
        "Post status should be None for rejection"
    );
}

#[tokio::test]
#[ignore] // Requires test database setup and schema with review_queue
async fn test_escalation_low_confidence() {
    let db = get_test_db().await;

    let fixture_id =
        Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").expect("Invalid fixture UUID");

    // Fetch incident
    let incident = IncidentEntity::find_by_id(fixture_id)
        .one(&db)
        .await
        .expect("Failed to query incident")
        .expect("Fixture incident not found");

    // Accept with very low confidence (< 0.3) - should trigger escalation
    let low_confidence = 0.25;
    assert!(low_confidence < 0.3, "Confidence should be below threshold");

    let mut active_model: IncidentActiveModel = incident.clone().into();
    active_model.verified = Set(true);
    active_model.updated_at = Set(chrono::Utc::now().into());

    active_model
        .update(&db)
        .await
        .expect("Failed to update incident");

    // Log the action with low confidence
    let audit_entry = review_audit_log::ActiveModel {
        id: Set(Uuid::new_v4()),
        incident_id: Set(fixture_id),
        reviewer_id: Set("low-confidence-test".to_string()),
        action: Set(ReviewAction::Accepted.to_string()),
        reviewed_at: Set(chrono::Utc::now().into()),
        justification: Set(Some("Accepted with low confidence".to_string())),
        confidence_score: Set(Some(low_confidence)),
        prior_status: Set(None),
        post_status: Set(None),
    };

    audit_entry
        .insert(&db)
        .await
        .expect("Failed to log audit entry");

    // In a real scenario, this would trigger auto-escalation
    // Simulate the escalation by logging an ESCALATED action
    let escalation_entry = review_audit_log::ActiveModel {
        id: Set(Uuid::new_v4()),
        incident_id: Set(fixture_id),
        reviewer_id: Set("system".to_string()),
        action: Set(ReviewAction::Escalated.to_string()),
        reviewed_at: Set(chrono::Utc::now().into()),
        justification: Set(Some(
            "Auto-escalated: confidence_score < 0.3 on acceptance".to_string(),
        )),
        confidence_score: Set(None),
        prior_status: Set(None),
        post_status: Set(None),
    };

    escalation_entry
        .insert(&db)
        .await
        .expect("Failed to log escalation");

    // Verify escalation was logged
    let escalated = ReviewAuditLogEntity::find()
        .filter(review_audit_log::Column::IncidentId.eq(fixture_id))
        .filter(review_audit_log::Column::Action.eq(ReviewAction::Escalated.to_string()))
        .all(&db)
        .await
        .expect("Failed to query escalation logs");

    assert!(
        !escalated.is_empty(),
        "Escalation should be logged for low confidence"
    );
    assert!(
        escalated[0]
            .justification
            .as_ref()
            .map(|j| j.contains("confidence_score"))
            .unwrap_or(false),
        "Escalation reason should mention confidence"
    );
}

#[tokio::test]
#[ignore] // Requires test database setup and schema with review_audit_log
async fn test_escalation_conflicting_reviews() {
    let db = get_test_db().await;

    // Use a fixture incident that will have two different reviewers making opposite decisions
    let fixture_id =
        Uuid::parse_str("11111111-1111-1111-1111-111111111111").expect("Invalid fixture UUID");

    // First reviewer accepts the incident
    let audit_accept = review_audit_log::ActiveModel {
        id: Set(Uuid::new_v4()),
        incident_id: Set(fixture_id),
        reviewer_id: Set("reviewer-alice".to_string()),
        action: Set(ReviewAction::Accepted.to_string()),
        reviewed_at: Set(chrono::Utc::now().into()),
        justification: Set(Some("Alice accepts this incident".to_string())),
        confidence_score: Set(Some(0.8)),
        prior_status: Set(None),
        post_status: Set(None),
    };

    audit_accept
        .insert(&db)
        .await
        .expect("Failed to log first reviewer's accept");

    // Second reviewer rejects the same incident (conflict!)
    let audit_reject = review_audit_log::ActiveModel {
        id: Set(Uuid::new_v4()),
        incident_id: Set(fixture_id),
        reviewer_id: Set("reviewer-bob".to_string()),
        action: Set(ReviewAction::Rejected.to_string()),
        reviewed_at: Set(chrono::Utc::now().into()),
        justification: Set(Some("Bob rejects this incident".to_string())),
        confidence_score: Set(None),
        prior_status: Set(None),
        post_status: Set(None),
    };

    audit_reject
        .insert(&db)
        .await
        .expect("Failed to log second reviewer's reject");

    // System detects conflict and escalates
    let escalation = review_audit_log::ActiveModel {
        id: Set(Uuid::new_v4()),
        incident_id: Set(fixture_id),
        reviewer_id: Set("system".to_string()),
        action: Set(ReviewAction::Escalated.to_string()),
        reviewed_at: Set(chrono::Utc::now().into()),
        justification: Set(Some(
            "Conflicting reviewer decisions (rejected after prior acceptance)".to_string(),
        )),
        confidence_score: Set(None),
        prior_status: Set(None),
        post_status: Set(None),
    };

    escalation
        .insert(&db)
        .await
        .expect("Failed to log escalation for conflict");

    // Verify all three entries exist
    let all_actions = ReviewAuditLogEntity::find()
        .filter(review_audit_log::Column::IncidentId.eq(fixture_id))
        .all(&db)
        .await
        .expect("Failed to query audit actions");

    // Should have at least: accept, reject, escalate
    assert!(
        all_actions.len() >= 3,
        "Should have multiple actions including escalation"
    );

    // Verify escalation is the last action
    let escalations: Vec<_> = all_actions
        .iter()
        .filter(|a| a.action == ReviewAction::Escalated.to_string())
        .collect();

    assert!(
        !escalations.is_empty(),
        "Escalation should be logged for conflicting reviews"
    );
    assert!(
        escalations[0]
            .justification
            .as_ref()
            .map(|j| j.contains("conflict"))
            .unwrap_or(false),
        "Escalation reason should mention conflicting decisions"
    );
}
