//! # Serveur de collecte Winlog (Rust/Axum/SQLite)
//!
//! Serveur HTTP pour la collecte d'événements de monitoring (connexions/déconnexions/matériel).
//! Implémente la même logique que le serveur PHP mais avec des performances améliorées.
//!
//! ## Architecture
//! - **Framework Web** : Axum 0.7 (simplicité + stabilité)
//! - **Base de données** : SQLite avec SQLx (structure partitionnée events_today/events_history)
//! - **Sérialisation** : serde + serde_json
//! - **Logging** : tracing + tracing-subscriber
//!
//! ## Endpoints
//! - `POST /api/v1/events` - Collecte d'événements (logique principale)
//! - `GET /api/v1/sessions/current` - Liste des sessions actuellement ouvertes
//! - `GET /health` - Health check
//!
//! ## Configuration
//! Le serveur charge sa configuration depuis `config.toml` au démarrage.

mod config;
mod models;
mod database;
mod handlers;
mod queries;  // Module contenant toutes les requêtes SQL

use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    config::Config,
    database::Database,
    handlers::{AppState, collect_event, health_check, get_current_sessions},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Chargement de la configuration
    let config = Config::from_file("config.toml")
        .expect("Impossible de charger config.toml");

    config.validate()
        .expect("Configuration invalide");

    // 2. Initialisation du logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=info", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("🚀 Démarrage du serveur Winlog...");
    tracing::info!("Configuration chargée depuis config.toml");

    // 3. Connexion à la base de données SQLite
    tracing::info!("Connexion à la base SQLite: {}", config.database.path_buf().display());
    let db = Database::new(&config.database)
        .await
        .expect("Impossible de se connecter à la base SQLite");
    tracing::info!("✓ Connexion SQLite établie");

    // 4. Création de l'état partagé
    let state = AppState {
        config: config.clone(),
        db,
    };

    // 5. Définition des routes Axum
    let app = Router::new()
        // Route principale : collecte d'événements
        .route("/api/v1/events", post(collect_event))
        
        // Liste des sessions ouvertes
        .route("/api/v1/sessions/current", get(get_current_sessions))
        
        // Health check
        .route("/health", get(health_check))
        
        // État partagé
        .with_state(state)
        
        // Middleware de logging HTTP
        .layer(TraceLayer::new_for_http());

    // 6. Démarrage du serveur
    let addr: SocketAddr = config.bind_address().parse()?;
    
    tracing::info!("✓ Serveur Winlog démarré sur http://{}", addr);
    tracing::info!("  POST /api/v1/events            - Collecte d'événements");
    tracing::info!("  GET  /api/v1/sessions/current  - Sessions ouvertes");
    tracing::info!("  GET  /health                   - Health check");
    tracing::info!("");
    tracing::info!("Appuyez sur Ctrl+C pour arrêter le serveur");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;

    Ok(())
}

/// Signal de shutdown gracieux (Ctrl+C)
async fn shutdown_signal() {
    match tokio::signal::ctrl_c().await {
        Ok(()) => {
            tracing::info!("");
            tracing::info!("🛑 Arrêt gracieux du serveur...");
        }
        Err(e) => {
            tracing::error!("Erreur lors de l'installation du handler Ctrl+C: {}", e);
            tracing::info!("🛑 Arrêt du serveur...");
        }
    }
}
