//! # Configuration centralisée du projet Winlog
//!
//! Ce module gère la configuration des clients Winlog avec une approche hiérarchique :
//! 
//! ## Hiérarchie de configuration
//! 
//! 1. **Variables d'environnement** (priorité haute)
//! 2. **Constantes par défaut** (fallback si variables absentes)
//! 
//! ## Variables d'environnement supportées
//! 
//! | Variable | Type | Défaut | Description |
//! |----------|------|--------|-------------|
//! | `WINLOG_SERVER_URL` | String | `http://127.0.0.1:3000/api/v1/events` | URL du serveur de monitoring |
//! | `WINLOG_TIMEOUT` | u64 | `30` | Timeout HTTP en secondes |
//! | `WINLOG_MAX_RETRIES` | u32 | `3` | Nombre de tentatives max |
//! | `WINLOG_RETRY_DELAY_MS` | u64 | `1000` | Délai entre retries (ms) |
//! | `WINLOG_USER_AGENT` | String | `Winlog/0.1.0` | User-Agent HTTP |
//! 
//! ## Déploiement en production
//! 
//! ### Windows (via GPO)
//! 
//! ```powershell
//! # Configuration système (appliquée à toutes les sessions)
//! [System.Environment]::SetEnvironmentVariable(
//!     "WINLOG_SERVER_URL", 
//!     "http://192.168.1.100:3000/api/v1/events", 
//!     "Machine"
//! )
//! ```
//! 
//! **GPO** : Computer Configuration > Preferences > Windows Settings > Environment
//! - Variable : `WINLOG_SERVER_URL`
//! - Value : `http://192.168.1.100:3000/api/v1/events`
//! - Action : Create/Update
//! 
//! ### Linux (PAM/systemd)
//! 
//! **Option 1 - /etc/environment** (recommandé) :
//! ```bash
//! echo 'WINLOG_SERVER_URL=http://192.168.1.100:3000/api/v1/events' >> /etc/environment
//! ```
//! 
//! **Option 2 - /etc/profile.d** :
//! ```bash
//! cat > /etc/profile.d/winlog.sh <<EOF
//! export WINLOG_SERVER_URL=http://192.168.1.100:3000/api/v1/events
//! export WINLOG_TIMEOUT=30
//! EOF
//! chmod +x /etc/profile.d/winlog.sh
//! ```
//! 
//! **Option 3 - Systemd service** (si exécuté via service) :
//! ```ini
//! [Service]
//! Environment="WINLOG_SERVER_URL=http://192.168.1.100:3000/api/v1/events"
//! ```
//! 
//! ## Avantages de cette approche
//! 
//! - ✅ **Pas de recompilation** : Changement de config sans rebuild
//! - ✅ **Déploiement centralisé** : GPO Windows / Ansible Linux
//! - ✅ **Sécurité raisonnable** : Variables auditables via GPO
//! - ✅ **Flexibilité** : Configuration différente par environnement
//! - ✅ **Maintenance simplifiée** : Un seul binaire pour tous les environnements

use std::env;

// ============================================================================
// CONSTANTES PAR DÉFAUT (utilisées si variables d'environnement absentes)
// ============================================================================

/// URL par défaut du serveur de monitoring (tests locaux)
const DEFAULT_SERVER_URL: &str = "http://127.0.0.1:3000/api/v1/events";

/// Timeout par défaut pour les requêtes HTTP (en secondes)
const DEFAULT_TIMEOUT: u64 = 30;

/// Nombre maximum de tentatives de retry
const DEFAULT_MAX_RETRIES: u32 = 3;

/// Délai entre les tentatives de retry (en millisecondes)
const DEFAULT_RETRY_DELAY_MS: u64 = 1000;

/// User-Agent par défaut utilisé pour les requêtes HTTP
const DEFAULT_USER_AGENT: &str = "Winlog/0.1.0";

// ============================================================================
// FONCTIONS D'ACCÈS À LA CONFIGURATION (lecture avec fallback)
// ============================================================================

/// Récupère l'URL du serveur de monitoring
///
/// **Priorité** :
/// 1. Variable d'environnement `WINLOG_SERVER_URL`
/// 2. Constante par défaut (`http://127.0.0.1:3000/api/v1/events`)
///
/// # Exemples
///
/// ```bash
/// # Linux
/// export WINLOG_SERVER_URL=http://192.168.1.100:3000/api/v1/events
/// ./logon
/// ```
///
/// ```powershell
/// # Windows
/// $env:WINLOG_SERVER_URL = "http://192.168.1.100:3000/api/v1/events"
/// .\logon.exe
/// ```
pub fn server_url() -> String {
    env::var("WINLOG_SERVER_URL")
        .unwrap_or_else(|_| DEFAULT_SERVER_URL.to_string())
}

/// Récupère le timeout HTTP (en secondes)
///
/// **Priorité** :
/// 1. Variable d'environnement `WINLOG_TIMEOUT` (doit être un entier valide)
/// 2. Constante par défaut (`30`)
///
/// # Exemples
///
/// ```bash
/// export WINLOG_TIMEOUT=60
/// ./logon
/// ```
pub fn timeout() -> u64 {
    env::var("WINLOG_TIMEOUT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_TIMEOUT)
}

/// Récupère le nombre maximum de tentatives de retry
///
/// **Priorité** :
/// 1. Variable d'environnement `WINLOG_MAX_RETRIES` (doit être un entier valide)
/// 2. Constante par défaut (`3`)
///
/// # Exemples
///
/// ```bash
/// export WINLOG_MAX_RETRIES=5
/// ./logon
/// ```
pub fn max_retries() -> u32 {
    env::var("WINLOG_MAX_RETRIES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_MAX_RETRIES)
}

/// Récupère le délai entre les retries (en millisecondes)
///
/// **Priorité** :
/// 1. Variable d'environnement `WINLOG_RETRY_DELAY_MS` (doit être un entier valide)
/// 2. Constante par défaut (`1000`)
///
/// # Exemples
///
/// ```bash
/// export WINLOG_RETRY_DELAY_MS=2000
/// ./logon
/// ```
pub fn retry_delay_ms() -> u64 {
    env::var("WINLOG_RETRY_DELAY_MS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_RETRY_DELAY_MS)
}

/// Récupère le User-Agent HTTP
///
/// **Priorité** :
/// 1. Variable d'environnement `WINLOG_USER_AGENT`
/// 2. Constante par défaut (`Winlog/0.1.0`)
///
/// # Exemples
///
/// ```bash
/// export WINLOG_USER_AGENT="Winlog/0.2.0 (Production)"
/// ./logon
/// ```
pub fn user_agent() -> String {
    env::var("WINLOG_USER_AGENT")
        .unwrap_or_else(|_| DEFAULT_USER_AGENT.to_string())
}