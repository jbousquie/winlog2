//! # Module des modèles de données
//!
//! Définit les structures de données échangées entre le client et le serveur,
//! ainsi que les modèles de la base de données.

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Événement reçu du client (payload JSON)
#[derive(Debug, Clone, Deserialize)]
pub struct ClientEvent {
    /// Nom d'utilisateur
    pub username: String,
    
    /// Code d'action : 'C' (connexion), 'D' (déconnexion), 'M' (matériel)
    pub action: String,
    
    /// Timestamp de l'événement (ISO 8601)
    pub timestamp: String,
    
    /// Nom de la machine (optionnel)
    pub hostname: Option<String>,
    
    /// Informations système d'exploitation
    pub os_info: Option<OsInfo>,
    
    /// Informations matérielles (pour action='M' uniquement)
    pub hardware_info: Option<serde_json::Value>,
}

/// Informations système d'exploitation
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OsInfo {
    pub os_name: Option<String>,
    pub os_version: Option<String>,
    pub kernel_version: Option<String>,
}

/// Événement stocké en base de données (utilisé pour les requêtes SELECT futures)
#[allow(dead_code)]  // Prévu pour API de consultation
#[derive(Debug, Clone, FromRow)]
pub struct DbEvent {
    pub id: i64,
    pub username: String,
    pub action: String,
    pub timestamp: String,
    pub hostname: Option<String>,
    pub source_ip: Option<String>,
    pub server_timestamp: String,
    pub os_name: Option<String>,
    pub os_version: Option<String>,
    pub kernel_version: Option<String>,
    pub hardware_info: Option<String>,
    pub session_uuid: String,
    pub created_at: String,
}

/// Session ouverte trouvée en base
#[derive(Debug, Clone, FromRow)]
pub struct OpenSession {
    pub session_uuid: String,
    #[allow(dead_code)]  // Timestamp disponible mais non utilisé actuellement
    pub timestamp: String,
}

/// Session en cours pour l'API GET /api/v1/sessions/current
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct CurrentSession {
    /// Nom d'utilisateur
    pub username: String,
    
    /// Nom de la machine (nullable en base)
    pub hostname: Option<String>,
    
    /// Date/heure de connexion (ISO 8601)
    pub connected_at: String,
    
    /// Identifiant unique de session
    pub session_uuid: String,
    
    /// Adresse IP source (nullable en base)
    pub source_ip: Option<String>,
    
    /// Nom du système d'exploitation (nullable en base)
    pub os_name: Option<String>,
    
    /// Version du système d'exploitation (nullable en base)
    pub os_version: Option<String>,
}

/// Réponse de succès retournée au client
#[derive(Debug, Serialize)]
pub struct SuccessResponse {
    pub status: String,
    pub message: String,
    pub event_id: i64,
    pub session_uuid: String,
    pub action: String,
    pub username: String,
}

/// Réponse d'erreur retournée au client
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ErrorResponse {
    /// Crée une réponse d'erreur simple
    pub fn new(error: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            details: None,
        }
    }

    /// Crée une réponse d'erreur avec détails (prévu pour future gestion d'erreurs avancée)
    #[allow(dead_code)]  // API complète pour évolutions futures
    pub fn with_details(error: impl Into<String>, details: serde_json::Value) -> Self {
        Self {
            error: error.into(),
            details: Some(details),
        }
    }
}
