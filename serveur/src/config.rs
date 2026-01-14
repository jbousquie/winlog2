//! # Module de configuration
//!
//! Charge et valide la configuration depuis le fichier `config.toml`.
//! Utilise serde pour désérialiser automatiquement le TOML en structures Rust.

use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

/// Configuration complète du serveur
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub security: SecurityConfig,
    #[allow(dead_code)]  // Logging prévu pour évolutions futures
    pub logging: LoggingConfig,
}

/// Configuration du serveur HTTP
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    /// Adresse d'écoute (ex: "127.0.0.1")
    pub host: String,
    /// Port d'écoute (ex: 3000)
    pub port: u16,
}

/// Configuration de la base de données SQLite
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    /// Chemin vers le fichier .db (sera converti en PathBuf)
    pub path: String,
    /// Mode journal (WAL recommandé)
    pub pragma_journal_mode: String,
    /// Synchronisation (NORMAL recommandé)
    pub pragma_synchronous: String,
    /// Timeout d'attente en ms
    pub pragma_busy_timeout: u32,
    /// Taille du cache (nombre de pages)
    pub pragma_cache_size: i32,
}

impl DatabaseConfig {
    /// Retourne le chemin de la base en PathBuf (multi-plateforme)
    ///
    /// Convertit le chemin TOML en PathBuf natif du système d'exploitation.
    /// Gère automatiquement les séparateurs Windows (\) et Unix (/).
    pub fn path_buf(&self) -> PathBuf {
        PathBuf::from(&self.path)
    }

    /// Retourne l'URL SQLite complète pour SQLx
    ///
    /// Format : "sqlite://chemin/vers/base.db" ou "sqlite:chemin/vers/base.db"
    /// SQLx accepte les deux syntaxes. On utilise la forme simple sans "//".
    pub fn sqlite_url(&self) -> String {
        let path_buf = self.path_buf();
        // SQLx accepte les chemins relatifs et absolus
        // PathBuf gère automatiquement les séparateurs Windows/Linux
        format!("sqlite:{}", path_buf.display())
    }
}

/// Configuration de sécurité
#[derive(Debug, Clone, Deserialize)]
pub struct SecurityConfig {
    /// User-Agent attendu des clients
    pub expected_user_agent: String,
    /// Actions autorisées (C, D, M)
    pub valid_actions: Vec<String>,
}

/// Configuration du logging (prévu pour personnalisation future)
#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    /// Niveau de log (trace, debug, info, warn, error)
    #[allow(dead_code)]  // API complète pour évolutions futures
    pub level: String,
    /// Format (compact, full)
    #[allow(dead_code)]  // API complète pour évolutions futures
    pub format: String,
}

impl Config {
    /// Charge la configuration depuis un fichier TOML
    ///
    /// # Arguments
    /// * `path` - Chemin vers le fichier config.toml
    ///
    /// # Erreurs
    /// Retourne une erreur si le fichier n'existe pas ou si le format est invalide
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path.as_ref())
            .map_err(|e| ConfigError::FileRead(e.to_string()))?;

        toml::from_str(&content)
            .map_err(|e| ConfigError::Parse(e.to_string()))
    }

    /// Retourne l'adresse complète d'écoute (host:port)
    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }

    /// Valide que les actions configurées sont valides
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Vérifier que les actions sont valides
        for action in &self.security.valid_actions {
            if !["C", "D", "M"].contains(&action.as_str()) {
                return Err(ConfigError::InvalidAction(action.clone()));
            }
        }

        // Vérifier que le port est dans une plage valide
        if self.server.port == 0 {
            return Err(ConfigError::InvalidPort);
        }

        Ok(())
    }
}

/// Erreurs de configuration
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Impossible de lire le fichier de configuration: {0}")]
    FileRead(String),

    #[error("Erreur de parsing TOML: {0}")]
    Parse(String),

    #[error("Action invalide: {0}")]
    InvalidAction(String),

    #[error("Port invalide")]
    InvalidPort,
}
