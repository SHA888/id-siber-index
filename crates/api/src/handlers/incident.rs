//! Incident API handlers

use axum::extract::{Path, Query};
use axum::response::Json;
use uuid::Uuid;

use crate::error::ApiError;
use schema::models::incident::{CreateIncident, IncidentSearchParams, UpdateIncident};

/// Get all incidents with optional filtering
pub async fn get_incidents(
    Query(_params): Query<IncidentSearchParams>,
) -> Result<Json<Vec<schema::models::incident::Incident>>, ApiError> {
    // TODO: Implement incident retrieval with filtering
    Ok(Json(vec![]))
}

/// Get a specific incident by ID
pub async fn get_incident(
    Path(_id): Path<Uuid>,
) -> Result<Json<schema::models::incident::Incident>, ApiError> {
    // TODO: Implement incident retrieval by ID
    Err(ApiError::NotFound("Incident not found".to_string()))
}

/// Create a new incident
pub async fn create_incident(
    Json(_incident): Json<CreateIncident>,
) -> Result<Json<schema::models::incident::Incident>, ApiError> {
    // TODO: Implement incident creation
    Err(ApiError::Internal("Not implemented".to_string()))
}

/// Update an existing incident
pub async fn update_incident(
    Path(_id): Path<Uuid>,
    Json(_incident): Json<UpdateIncident>,
) -> Result<Json<schema::models::incident::Incident>, ApiError> {
    // TODO: Implement incident update
    Err(ApiError::Internal("Not implemented".to_string()))
}

/// Delete an incident
pub async fn delete_incident(Path(_id): Path<Uuid>) -> Result<(), ApiError> {
    // TODO: Implement incident deletion
    Err(ApiError::Internal("Not implemented".to_string()))
}
