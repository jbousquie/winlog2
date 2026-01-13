//! # Module de base de données
//!
//! Gère la connexion SQLite, les requêtes SQL et la logique métier
//! liée aux sessions (fermeture auto, génération UUID, etc.).

use chrono::Utc;
use md5;
use sqlx::{SqlitePool, Row};
use crate::config::DatabaseConfig;
use crate::models::{ClientEvent, OpenSession};

/// Gestionnaire de base de données
#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// Initialise la connexion à la base SQLite avec configuration des PRAGMA
    ///
    /// # Arguments
    /// * `config` - Configuration de la base de données
    ///
    /// # Erreurs
    /// Retourne une erreur si la connexion échoue
    pub async fn new(config: &DatabaseConfig) -> Result<Self, sqlx::Error> {
        let pool = SqlitePool::connect(&config.path).await?;

        // Configuration des PRAGMA SQLite
        let pragmas = vec![
            format!("PRAGMA journal_mode = {}", config.pragma_journal_mode),
            format!("PRAGMA synchronous = {}", config.pragma_synchronous),
            format!("PRAGMA busy_timeout = {}", config.pragma_busy_timeout),
            format!("PRAGMA cache_size = {}", config.pragma_cache_size),
        ];

        for pragma in pragmas {
            sqlx::query(&pragma).execute(&pool).await?;
        }

        Ok(Self { pool })
    }

    /// Génère un identifiant de session unique
    ///
    /// Format : username@hostname@hash6
    /// Hash = MD5(username + hostname + date + timestamp_nanos)
    ///
    /// # Arguments
    /// * `username` - Nom d'utilisateur
    /// * `hostname` - Nom de la machine
    /// * `timestamp` - Timestamp ISO 8601
    pub fn generate_session_id(username: &str, hostname: &str, timestamp: &str) -> String {
        // Extraire la date (YYYY-MM-DD) - safe slice avec get()
        let date = timestamp.get(..10).unwrap_or("1970-01-01");
        
        // Créer un hash unique avec timestamp nano pour éviter collisions
        let now_nanos = Utc::now().timestamp_nanos_opt().unwrap_or(0);
        let data = format!("{}{}{}{}", username, hostname, date, now_nanos);
        let hash = format!("{:x}", md5::compute(data));
        
        // Prendre les 6 premiers caractères du hash - safe slice
        // MD5 produit toujours 32 caractères hex, donc .get(..6) ne peut jamais échouer
        let short_hash = hash.get(..6).unwrap_or("000000");
        
        format!("{}@{}@{}", username, hostname, short_hash)
    }

    /// Recherche une session ouverte aujourd'hui pour un utilisateur/machine
    ///
    /// # Arguments
    /// * `username` - Nom d'utilisateur
    /// * `hostname` - Nom de la machine
    /// * `timestamp` - Timestamp de référence
    ///
    /// # Retourne
    /// `Some(OpenSession)` si une session ouverte existe, `None` sinon
    pub async fn find_open_session_today(
        &self,
        username: &str,
        hostname: &str,
        timestamp: &str,
    ) -> Result<Option<OpenSession>, sqlx::Error> {
        let result = sqlx::query_as::<_, OpenSession>(
            r#"
            SELECT session_uuid, timestamp FROM events_today 
            WHERE username = ? AND hostname = ? AND action = 'C'
            AND DATE(timestamp) = DATE(?)
            AND NOT EXISTS (
                SELECT 1 FROM events_today e2 
                WHERE e2.session_uuid = events_today.session_uuid 
                AND e2.action = 'D'
            )
            ORDER BY timestamp DESC 
            LIMIT 1
            "#,
        )
        .bind(username)
        .bind(hostname)
        .bind(timestamp)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    /// Trouve la dernière session ouverte (pour associer une déconnexion)
    ///
    /// # Arguments
    /// * `username` - Nom d'utilisateur
    /// * `hostname` - Nom de la machine
    ///
    /// # Retourne
    /// `Some(session_uuid)` si trouvée, `None` sinon
    pub async fn find_last_open_session(
        &self,
        username: &str,
        hostname: &str,
    ) -> Result<Option<String>, sqlx::Error> {
        let result = sqlx::query(
            r#"
            SELECT session_uuid FROM events_today 
            WHERE username = ? AND hostname = ? AND action = 'C'
            AND NOT EXISTS (
                SELECT 1 FROM events_today e2 
                WHERE e2.session_uuid = events_today.session_uuid 
                AND e2.action = 'D'
            )
            ORDER BY timestamp DESC 
            LIMIT 1
            "#,
        )
        .bind(username)
        .bind(hostname)
        .fetch_optional(&self.pool)
        .await?;

        // Utilisation de try_get au lieu de get pour éviter panic si colonne manquante
        Ok(result.and_then(|row| row.try_get::<String, _>("session_uuid").ok()))
    }

    /// Insère une déconnexion automatique (pour fermer une session orpheline)
    ///
    /// # Arguments
    /// * `event` - Événement de connexion qui provoque la fermeture
    /// * `session_uuid` - UUID de la session à fermer
    /// * `source_ip` - Adresse IP source
    pub async fn insert_auto_disconnect(
        &self,
        event: &ClientEvent,
        session_uuid: &str,
        source_ip: &str,
    ) -> Result<(), sqlx::Error> {
        // Timestamp 1 seconde avant la nouvelle connexion
        let disconnect_time = chrono::DateTime::parse_from_rfc3339(&event.timestamp)
            .map(|dt| dt - chrono::Duration::seconds(1))
            .unwrap_or_else(|_| Utc::now().into())
            .to_rfc3339();

        let server_timestamp = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO events_today (
                username, action, timestamp, hostname, source_ip, server_timestamp,
                os_name, os_version, kernel_version, session_uuid
            ) VALUES (?, 'D', ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&event.username)
        .bind(&disconnect_time)
        .bind(event.hostname.as_deref())
        .bind(source_ip)
        .bind(&server_timestamp)
        .bind(event.os_info.as_ref().and_then(|os| os.os_name.as_deref()))
        .bind(event.os_info.as_ref().and_then(|os| os.os_version.as_deref()))
        .bind(event.os_info.as_ref().and_then(|os| os.kernel_version.as_deref()))
        .bind(session_uuid)
        .execute(&self.pool)
        .await?;

        tracing::info!("Déconnexion automatique insérée pour session: {}", session_uuid);
        Ok(())
    }

    /// Insère un nouvel événement dans events_today
    ///
    /// # Arguments
    /// * `event` - Événement client
    /// * `session_uuid` - UUID de session généré
    /// * `source_ip` - Adresse IP source
    ///
    /// # Retourne
    /// L'ID de l'événement inséré
    pub async fn insert_event(
        &self,
        event: &ClientEvent,
        session_uuid: &str,
        source_ip: &str,
    ) -> Result<i64, sqlx::Error> {
        let server_timestamp = Utc::now().to_rfc3339();

        // Sérialiser hardware_info si présent
        let hardware_json = event.hardware_info.as_ref()
            .map(|hw| serde_json::to_string(hw).ok())
            .flatten();

        let result = sqlx::query(
            r#"
            INSERT INTO events_today (
                username, action, timestamp, hostname, source_ip, server_timestamp,
                os_name, os_version, kernel_version, hardware_info, session_uuid
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&event.username)
        .bind(&event.action)
        .bind(&event.timestamp)
        .bind(event.hostname.as_deref())
        .bind(source_ip)
        .bind(&server_timestamp)
        .bind(event.os_info.as_ref().and_then(|os| os.os_name.as_deref()))
        .bind(event.os_info.as_ref().and_then(|os| os.os_version.as_deref()))
        .bind(event.os_info.as_ref().and_then(|os| os.kernel_version.as_deref()))
        .bind(hardware_json.as_deref())
        .bind(session_uuid)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }
}
