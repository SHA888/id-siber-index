//! Incident routes

use axum::{Router, routing::get};

use crate::handlers::incident::*;

pub fn routes() -> Router {
    Router::new()
        .route("/", get(get_incidents).post(create_incident))
        .route(
            "/:id",
            get(get_incident)
                .put(update_incident)
                .delete(delete_incident),
        )
}
