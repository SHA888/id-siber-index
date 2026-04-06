//! Search API handlers

use axum::extract::Query;
use axum::response::Json;

use crate::error::ApiError;
use schema::models::incident::IncidentSearchParams;

/// Search incidents
pub async fn search_incidents(
    Query(params): Query<IncidentSearchParams>,
) -> Result<Json<Vec<schema::models::incident::Incident>>, ApiError> {
    // TODO: Implement incident search
    Ok(Json(vec![]))
}
