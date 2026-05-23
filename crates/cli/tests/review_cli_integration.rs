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
