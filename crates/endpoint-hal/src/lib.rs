pub mod error;
pub mod hal;
pub mod journal;
pub mod state;

use axum::Router;

use crate::state::AppState;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/journals", axum::routing::post(journal::create_journal))
        .route("/journals", axum::routing::get(journal::list_journals))
        .route("/journals/{id}", axum::routing::get(journal::get_journal))
        .route(
            "/journals/{id}",
            axum::routing::patch(journal::update_journal),
        )
        .route(
            "/journals/{id}",
            axum::routing::delete(journal::delete_journal),
        )
        .with_state(state)
}
