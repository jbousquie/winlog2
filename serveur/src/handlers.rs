//! # Module des handlers HTTP
//!
//! Définit les endpoints de l'API REST pour la collecte des événements.
//! Implémente la même logique que le serveur PHP index.php.

use axum::{
    extract::{State, ConnectInfo},
    http::{StatusCode, HeaderMap, header},
    Json,
    response::IntoResponse,
};
use std::net::SocketAddr;
use crate::{
    config::Config,
    database::Database,
    models::{ClientEvent, SuccessResponse, ErrorResponse},
};

/// État partagé de l'application
#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db: Database,
}

/// Handler principal : collecte d'événements (POST /api/v1/events)
///
/// Correspond à la logique de serveur/php/index.php :
/// 1. Validation User-Agent et Content-Type
/// 2. Validation de la structure JSON
/// 3. Traitement selon l'action (C/D/M)
/// 4. Insertion en base (events_today)
/// 5. Retour réponse JSON
pub async fn collect_event(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(event): Json<ClientEvent>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    
    // 1. Validation User-Agent
    let user_agent = headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if !user_agent.starts_with(&state.config.security.expected_user_agent) {
        tracing::warn!("Invalid User-Agent: {}", user_agent);
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse::new("Invalid User-Agent")),
        ));
    }

    // 2. Validation de la structure JSON
    if event.username.is_empty() || event.action.is_empty() || event.timestamp.is_empty() {
        tracing::warn!("Invalid JSON structure: missing required fields");
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("Invalid JSON structure: missing required fields")),
        ));
    }

    // 3. Validation de l'action
    if !state.config.security.valid_actions.contains(&event.action) {
        tracing::warn!("Invalid action: {}", event.action);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(format!("Invalid action: {}", event.action))),
        ));
    }

    // 4. Validation du timestamp (format ISO 8601)
    if !event.timestamp.contains('T') {
        tracing::warn!("Invalid timestamp format: {}", event.timestamp);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("Invalid timestamp format (expected ISO 8601)")),
        ));
    }

    // 5. Extraction de l'adresse IP source
    let source_ip = extract_real_ip(headers.clone(), addr);

    // Log de réception
    tracing::info!(
        "Received event: username={}, action={}, hostname={:?}, ip={}",
        event.username,
        event.action,
        event.hostname,
        source_ip
    );

    // 6. Traitement selon l'action
    let session_uuid = match event.action.as_str() {
        "C" => handle_connection(&state, &event, &source_ip).await?,
        "D" => handle_disconnection(&state, &event).await?,
        "M" => handle_hardware(&state, &event).await?,
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("Unknown action")),
            ))
        }
    };

    // 7. Insertion de l'événement en base
    let event_id = state.db.insert_event(&event, &session_uuid, &source_ip)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("Database error")),
            )
        })?;

    // 8. Log de succès
    tracing::info!(
        "Data stored: ID={} - {} - {} - Session: {} from {}",
        event_id,
        event.username,
        event.action,
        session_uuid,
        source_ip
    );

    // 9. Réponse de succès
    Ok(Json(SuccessResponse {
        status: "success".to_string(),
        message: "Data stored in database".to_string(),
        event_id,
        session_uuid,
        action: event.action.clone(),
        username: event.username.clone(),
    }))
}

/// Traite une connexion (action='C')
///
/// Logique :
/// 1. Chercher si une session est ouverte aujourd'hui
/// 2. Si oui, la fermer automatiquement (déconnexion auto)
/// 3. Générer un nouveau session_uuid
async fn handle_connection(
    state: &AppState,
    event: &ClientEvent,
    source_ip: &str,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let hostname = event.hostname.as_deref().unwrap_or("unknown");

    // Chercher session ouverte aujourd'hui
    let open_session = state.db
        .find_open_session_today(&event.username, hostname, &event.timestamp)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("Database error")),
            )
        })?;

    // Si session ouverte trouvée, la fermer automatiquement
    if let Some(session) = open_session {
        tracing::warn!(
            "Session ouverte détectée pour {}@{} - fermeture automatique",
            event.username,
            hostname
        );

        state.db
            .insert_auto_disconnect(event, &session.session_uuid, source_ip)
            .await
            .map_err(|e| {
                tracing::error!("Failed to insert auto-disconnect: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse::new("Database error")),
                )
            })?;
    }

    // Générer nouveau session_uuid
    Ok(Database::generate_session_id(&event.username, hostname, &event.timestamp))
}

/// Traite une déconnexion (action='D')
///
/// Logique :
/// 1. Chercher la dernière session ouverte
/// 2. Si trouvée, utiliser son UUID
/// 3. Sinon, générer un UUID "orphan_"
async fn handle_disconnection(
    state: &AppState,
    event: &ClientEvent,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let hostname = event.hostname.as_deref().unwrap_or("unknown");

    // Chercher dernière session ouverte
    let session_uuid = state.db
        .find_last_open_session(&event.username, hostname)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("Database error")),
            )
        })?;

    match session_uuid {
        Some(uuid) => Ok(uuid),
        None => {
            tracing::warn!(
                "Aucune session ouverte trouvée pour {}@{}",
                event.username,
                hostname
            );
            // UUID orphelin
            Ok(format!(
                "orphan_{}",
                Database::generate_session_id(&event.username, hostname, &event.timestamp)
            ))
        }
    }
}

/// Traite un événement matériel (action='M')
///
/// Logique : Génère simplement un UUID préfixé "hardware_"
async fn handle_hardware(
    _state: &AppState,
    event: &ClientEvent,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let hostname = event.hostname.as_deref().unwrap_or("unknown");
    Ok(format!(
        "hardware_{}",
        Database::generate_session_id(&event.username, hostname, &event.timestamp)
    ))
}

/// Extrait l'adresse IP réelle du client (support proxies/CDN)
///
/// Ordre de priorité (comme dans le PHP) :
/// 1. CF-Connecting-IP (Cloudflare)
/// 2. X-Forwarded-For (Load balancer/proxy)
/// 3. REMOTE_ADDR (direct)
fn extract_real_ip(headers: HeaderMap, addr: SocketAddr) -> String {
    // 1. Cloudflare
    if let Some(ip) = headers.get("cf-connecting-ip") {
        if let Ok(ip_str) = ip.to_str() {
            return ip_str.to_string();
        }
    }

    // 2. X-Forwarded-For
    if let Some(forwarded) = headers.get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            if let Some(first_ip) = forwarded_str.split(',').next() {
                return first_ip.trim().to_string();
            }
        }
    }

    // 3. Adresse directe
    addr.ip().to_string()
}

/// Health check endpoint (GET /health)
pub async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "service": "winlog-server",
        "version": env!("CARGO_PKG_VERSION")
    }))
}
