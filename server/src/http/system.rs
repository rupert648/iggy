use crate::configs::http::HttpMetricsConfig;
use crate::http::error::CustomError;
use crate::http::jwt::Identity;
use crate::http::mapper;
use crate::http::state::AppState;
use crate::streaming::session::Session;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Extension, Json, Router, body::StreamBody, http::header};
use iggy::models::client_info::{ClientInfo, ClientInfoDetails};
use iggy::models::stats::Stats;
use std::sync::Arc;
use tokio_util::io::ReaderStream;

const NAME: &str = "Iggy HTTP";
const PONG: &str = "pong";

pub fn router(state: Arc<AppState>, metrics_config: &HttpMetricsConfig) -> Router {
    let mut router = Router::new()
        .route("/", get(|| async { NAME }))
        .route("/ping", get(|| async { PONG }))
        .route("/stats", get(get_stats))
        .route("/clients", get(get_clients))
        .route("/clients/:client_id", get(get_client))
        .route("/snapshot", get(get_snapshot));
    if metrics_config.enabled {
        router = router.route(&metrics_config.endpoint, get(get_metrics));
    }

    router.with_state(state)
}

async fn get_metrics(State(state): State<Arc<AppState>>) -> Result<String, CustomError> {
    let system = state.system.read().await;
    Ok(system.metrics.get_formatted_output())
}

async fn get_stats(
    State(state): State<Arc<AppState>>,
    Extension(identity): Extension<Identity>,
) -> Result<Json<Stats>, CustomError> {
    let system = state.system.read().await;
    let stats = system
        .get_stats(&Session::stateless(identity.user_id))
        .await?;
    Ok(Json(stats))
}

async fn get_client(
    State(state): State<Arc<AppState>>,
    Extension(identity): Extension<Identity>,
    Path(client_id): Path<u32>,
) -> Result<Json<ClientInfoDetails>, CustomError> {
    let system = state.system.read().await;
    let client = system
        .get_client(&Session::stateless(identity.user_id), client_id)
        .await?;
    let client = client.read().await;
    let client = mapper::map_client(&client).await;
    Ok(Json(client))
}

async fn get_clients(
    State(state): State<Arc<AppState>>,
    Extension(identity): Extension<Identity>,
) -> Result<Json<Vec<ClientInfo>>, CustomError> {
    let system = state.system.read().await;
    let clients = system
        .get_clients(&Session::stateless(identity.user_id))
        .await?;
    let clients = mapper::map_clients(&clients).await;
    Ok(Json(clients))
}

async fn get_snapshot(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let system = state.system.read().await;
    let snapshot_file_name = system.create_snapshot_file().await?;

    let file = match tokio::fs::File::open(snapshot_file_name.clone()).await {
        Ok(file) => file,
        Err(err) => {
            // TODO: work out whats going on here
            return Err(CustomError::Error(iggy::error::Error::IoError(err)));
        }
    };

    let stream = ReaderStream::new(file);
    let body = StreamBody::new(stream);

    let headers = [
        (header::CONTENT_TYPE, String::from("text/toml; charset=utf-8")),
        (
            header::CONTENT_DISPOSITION,
            format!("attachment; filename={}", snapshot_file_name)
        ),
    ];

    Ok((headers, body))
}
