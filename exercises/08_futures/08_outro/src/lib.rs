// This is our last exercise. Let's go down a more unstructured path!
// Try writing an **asynchronous REST API** to expose the functionality
// of the ticket management system we built throughout the course.
// It should expose endpoints to:
//  - Create a ticket
//  - Retrieve ticket details
//  - Patch a ticket
//
// Use Rust's package registry, crates.io, to find the dependencies you need
// (if any) to build this system.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticket {
    id: Uuid,
    title: String,
    description: String,
    status: TicketStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TicketStatus {
    Open,
    InProgress,
    Closed,
}

#[derive(Debug, Deserialize)]
pub struct CreateTicketRequest {
    title: String,
    description: String,
}

#[derive(Debug, Deserialize)]
pub struct PatchTicketRequest {
    title: Option<String>,
    description: Option<String>,
    status: Option<TicketStatus>,
}

// Create simple in-memory store
use std::sync::{Arc, Mutex};

type TicketStore = Arc<Mutex<HashMap<Uuid, Ticket>>>;

fn create_store() -> TicketStore {
    Arc::new(Mutex::new(HashMap::new()))
}

// Implement handler functions
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};

// POST /tickets
async fn create_ticket(
    State(store): State<TicketStore>,
    Json(request): Json<CreateTicketRequest>,
) -> Result<Json<Ticket>, StatusCode> {
    let ticket = Ticket {
        id: Uuid::new_v4(),
        title: request.title,
        description: request.description,
        status: TicketStatus::Open,
    };

    match store.lock() {
        Ok(mut store) => {
            store.insert(ticket.id, ticket.clone());
            Ok(Json(ticket))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// GET /tickets/{id}
async fn get_ticket(
    State(store): State<TicketStore>,
    Path(id): Path<Uuid>,
) -> Result<Json<Ticket>, StatusCode> {
    match store.lock() {
        Ok(store) => match store.get(&id) {
            Some(ticket) => Ok(Json(ticket.clone())),
            None => Err(StatusCode::NOT_FOUND),
        },
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// PATCH /tickets/{id}
async fn patch_ticket(
    State(store): State<TicketStore>,
    Path(id): Path<Uuid>,
    Json(request): Json<PatchTicketRequest>,
) -> Result<Json<Ticket>, StatusCode> {
    let mut store = store.lock().unwrap();
    match store.get_mut(&id) {
        Some(ticket) => {
            if let Some(title) = request.title {
                ticket.title = title;
            }
            if let Some(description) = request.description {
                ticket.description = description;
            }
            if let Some(status) = request.status {
                ticket.status = status;
            }
            Ok(Json(ticket.clone()))
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

// Router and server
use axum::{
    routing::{get, patch, post},
    Router,
};

pub async fn run_server() {
    let store = create_store();

    let app = Router::new()
        .route("/tickets", post(create_ticket))
        .route("/tickets/:id", get(get_ticket))
        .route("/tickets/:id", patch(patch_ticket))
        .with_state(store);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("Server running on http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}

#[tokio::main]
async fn main() {
    run_server().await;
}
