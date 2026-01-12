//! # Configuration centralisée du projet Winlog
//!
//! Ce module contient toutes les constantes de configuration utilisées
//! par les différents binaires du projet.

/// Configuration par défaut du serveur de monitoring
pub const DEFAULT_SERVER_URL: &str = "http://192.168.122.1/winlog";

/// Timeout par défaut pour les requêtes HTTP (en secondes)
pub const DEFAULT_TIMEOUT: u64 = 30;

/// Nombre maximum de tentatives de retry
pub const MAX_RETRIES: u32 = 3;

/// User-Agent utilisé pour les requêtes HTTP
pub const USER_AGENT: &str = "Winlog/0.1.0 (Windows)";

/// Délai entre les tentatives de retry (en millisecondes)
pub const RETRY_DELAY_MS: u64 = 1000;