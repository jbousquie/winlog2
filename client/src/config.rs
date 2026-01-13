//! # Configuration centralisée du projet Winlog
//!
//! Ce module contient toutes les constantes de configuration utilisées
//! par les différents binaires du projet.

/// Configuration par défaut du serveur de monitoring
/// Pour tests locaux (client et serveur sur même machine) : 127.0.0.1:3000
/// Pour production : adresse IP du serveur distant (ex: 192.168.1.100:3000)
pub const DEFAULT_SERVER_URL: &str = "http://127.0.0.1:3000/api/v1/events";

/// Timeout par défaut pour les requêtes HTTP (en secondes)
pub const DEFAULT_TIMEOUT: u64 = 30;

/// Nombre maximum de tentatives de retry
pub const MAX_RETRIES: u32 = 3;

/// User-Agent utilisé pour les requêtes HTTP
/// Note : Accepte "(Windows)" ou "(Linux)" selon la plateforme
pub const USER_AGENT: &str = "Winlog/0.1.0";

/// Délai entre les tentatives de retry (en millisecondes)
pub const RETRY_DELAY_MS: u64 = 1000;