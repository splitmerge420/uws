use axum::{routing::post, Json, Router};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct IngestRequest {
    pub primary_package: String,
    pub lineage_hash: String,
    pub timestamp: i64,
}

#[derive(Serialize)]
pub struct IngestResponse {
    pub status: String,
}

async fn ingest(Json(_payload): Json<IngestRequest>) -> Json<IngestResponse> {
    Json(IngestResponse {
        status: "accepted".to_string(),
    })
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/v1/executions", post(ingest));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
