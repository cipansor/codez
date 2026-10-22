//! Codez backend: an Axum server exposing the Gitea-compatible `/api/v1`
//! surface plus a Leptos single-page frontend.
//!
//! The API layer is generated from Gitea's OpenAPI document (see
//! `scripts/gen_gitea_api.py`), so the route table and DTOs track upstream
//! rather than being maintained by hand.

mod api;
mod ids;
mod pagination;
mod respond;
mod seed;
mod state;
mod static_files;
mod time;
mod web;

#[cfg(test)]
mod tests;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;

use state::AppState;

/// Build the application router with the demo dataset loaded.
pub fn app() -> Router {
    let state: AppState = Arc::new(state::Stores::default());
    seed::load(&state);
    router(state)
}

/// Build the application router around an existing state.
pub fn router(state: AppState) -> Router {
    Router::new()
        .merge(api::routes::api_router())
        .merge(web::router())
        .fallback(static_files::serve)
        .with_state(state)
}

#[tokio::main]
async fn main() {
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);
    let address = SocketAddr::from(([0, 0, 0, 0], port));

    let listener = match tokio::net::TcpListener::bind(address).await {
        Ok(listener) => listener,
        Err(error) => {
            eprintln!("failed to bind {address}: {error}");
            std::process::exit(1);
        }
    };

    println!("codez listening on http://{address}");
    if let Err(error) = axum::serve(listener, app()).await {
        eprintln!("server error: {error}");
        std::process::exit(1);
    }
}
