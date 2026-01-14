//! # Module de requêtes SQL
//!
//! Centralise toutes les requêtes SQL utilisées par le serveur Winlog 2.
//!
//! ## Organisation
//!
//! - Toutes les requêtes sont des constantes publiques préfixées par `SQL_`
//! - Chaque constante est documentée avec :
//!   - **Objectif** : Pourquoi cette requête existe
//!   - **Logique** : Comment elle fonctionne (filtres, jointures, sous-requêtes)
//!   - **Paramètres** : Liste ordonnée des paramètres `?` avec leur signification
//!
//! ## Avantages de cette organisation
//!
//! - ✅ **Lisibilité** : Séparation claire entre logique métier et requêtes SQL
//! - ✅ **Maintenabilité** : Modification centralisée (une constante = tous les usages)
//! - ✅ **Documentation** : Commentaires détaillés pour chaque requête
//! - ✅ **Réutilisabilité** : Constantes facilement réutilisables dans tout le code
//! - ✅ **Testabilité** : Possibilité de tester les requêtes isolément
//!
//! ## Conventions de nommage
//!
//! - `SQL_FIND_*` : Requêtes de recherche (SELECT)
//! - `SQL_INSERT_*` : Requêtes d'insertion (INSERT)
//! - `SQL_UPDATE_*` : Requêtes de mise à jour (UPDATE)
//! - `SQL_DELETE_*` : Requêtes de suppression (DELETE)

// ============================================================================
// REQUÊTES DE RECHERCHE (SELECT)
// ============================================================================

/// Recherche une session de connexion (action='C') ouverte aujourd'hui pour un user@host donné.
/// 
/// **Objectif** : Éviter de créer plusieurs sessions le même jour pour le même utilisateur.
/// 
/// **Logique** :
/// - Filtre par username, hostname, action='C'
/// - Compare uniquement la date (DATE()) entre l'événement et le timestamp de référence
/// - Exclut les sessions déjà fermées (vérifie qu'il n'existe pas d'action='D' associée)
/// - Retourne la plus récente (ORDER BY timestamp DESC)
/// 
/// **Paramètres** :
/// - `?1` : username (TEXT)
/// - `?2` : hostname (TEXT)
/// - `?3` : timestamp de référence (TEXT ISO 8601) - pour extraction de la date
/// 
/// **Colonnes retournées** :
/// - `session_uuid` : Identifiant unique de la session
/// - `timestamp` : Date/heure de connexion (ISO 8601)
/// 
/// **Utilisé dans** : `database.rs::find_open_session_today()`
pub const SQL_FIND_OPEN_SESSION_TODAY: &str = r#"
    SELECT session_uuid, timestamp 
    FROM events_today 
    WHERE username = ? 
      AND hostname = ? 
      AND action = 'C'
      AND DATE(timestamp) = DATE(?)
      AND NOT EXISTS (
          SELECT 1 FROM events_today e2 
          WHERE e2.session_uuid = events_today.session_uuid 
            AND e2.action = 'D'
      )
    ORDER BY timestamp DESC 
    LIMIT 1
"#;

/// Recherche la dernière session de connexion (action='C') encore ouverte pour un user@host.
/// 
/// **Objectif** : Associer une déconnexion (action='D') orpheline à la bonne session.
/// 
/// **Logique** :
/// - Filtre par username, hostname, action='C'
/// - Exclut les sessions déjà fermées (vérifie qu'il n'existe pas d'action='D' associée)
/// - Retourne uniquement le session_uuid de la plus récente
/// - Pas de filtre de date : cherche dans toute la table events_today
/// 
/// **Paramètres** :
/// - `?1` : username (TEXT)
/// - `?2` : hostname (TEXT)
/// 
/// **Colonnes retournées** :
/// - `session_uuid` : Identifiant unique de la dernière session ouverte
/// 
/// **Utilisé dans** : `database.rs::find_last_open_session()`
pub const SQL_FIND_LAST_OPEN_SESSION: &str = r#"
    SELECT session_uuid 
    FROM events_today 
    WHERE username = ? 
      AND hostname = ? 
      AND action = 'C'
      AND NOT EXISTS (
          SELECT 1 FROM events_today e2 
          WHERE e2.session_uuid = events_today.session_uuid 
            AND e2.action = 'D'
      )
    ORDER BY timestamp DESC 
    LIMIT 1
"#;

// ============================================================================
// REQUÊTES D'INSERTION (INSERT)
// ============================================================================

/// Insère une déconnexion automatique (action='D') pour fermer une session orpheline.
/// 
/// **Objectif** : Fermer proprement une session qui n'a pas eu de logout explicite avant
///                une nouvelle connexion du même utilisateur.
/// 
/// **Contexte** : Lorsqu'un utilisateur se connecte alors qu'une session est déjà ouverte
///                (ex: reboot sans logout, crash système), le serveur insère automatiquement
///                une déconnexion fictive 1 seconde avant la nouvelle connexion.
/// 
/// **Logique** :
/// - Crée un événement de déconnexion avec action='D' (hardcodé dans la requête)
/// - Le timestamp est calculé comme : nouvelle_connexion - 1 seconde
/// - Réutilise les infos système (os_name, os_version, etc.) de la nouvelle connexion
/// - Associe la déconnexion à l'ancien session_uuid
/// 
/// **Paramètres** :
/// - `?1` : username (TEXT)
/// - `?2` : timestamp (TEXT ISO 8601) - calculé comme event.timestamp - 1 seconde
/// - `?3` : hostname (TEXT, nullable)
/// - `?4` : source_ip (TEXT) - IP du client
/// - `?5` : server_timestamp (TEXT ISO 8601) - timestamp serveur au moment de l'insertion
/// - `?6` : os_name (TEXT, nullable)
/// - `?7` : os_version (TEXT, nullable)
/// - `?8` : kernel_version (TEXT, nullable)
/// - `?9` : session_uuid (TEXT) - UUID de la session à fermer
/// 
/// **Note** : action='D' est hardcodé dans la requête (pas de paramètre)
/// 
/// **Utilisé dans** : `database.rs::insert_auto_disconnect()`
pub const SQL_INSERT_AUTO_DISCONNECT: &str = r#"
    INSERT INTO events_today (
        username, action, timestamp, hostname, source_ip, server_timestamp,
        os_name, os_version, kernel_version, session_uuid
    ) VALUES (?, 'D', ?, ?, ?, ?, ?, ?, ?, ?)
"#;

/// Insère un nouvel événement (connexion, déconnexion ou inventaire matériel) dans events_today.
/// 
/// **Objectif** : Enregistrer tous les événements client dans la table du jour.
/// 
/// **Logique** :
/// - Stocke l'événement brut avec toutes ses métadonnées
/// - timestamp = horodatage client (heure de l'événement côté client)
/// - server_timestamp = horodatage serveur (heure de réception)
/// - hardware_info = JSON optionnel (uniquement pour action='M' - inventaire matériel)
/// - session_uuid = identifiant de session généré côté serveur
/// 
/// **Paramètres** :
/// - `?1` : username (TEXT)
/// - `?2` : action (TEXT) - 'C' = connexion, 'D' = déconnexion, 'M' = inventaire matériel
/// - `?3` : timestamp (TEXT ISO 8601) - horodatage client
/// - `?4` : hostname (TEXT, nullable)
/// - `?5` : source_ip (TEXT) - adresse IP du client
/// - `?6` : server_timestamp (TEXT ISO 8601) - horodatage serveur
/// - `?7` : os_name (TEXT, nullable) - ex: "Windows", "Linux"
/// - `?8` : os_version (TEXT, nullable) - ex: "10.0.19045", "6.5.0-28-generic"
/// - `?9` : kernel_version (TEXT, nullable) - version du noyau
/// - `?10` : hardware_info (TEXT JSON, nullable) - infos matérielles sérialisées
/// - `?11` : session_uuid (TEXT) - identifiant de session généré
/// 
/// **Retourne** : L'ID de la ligne insérée (last_insert_rowid)
/// 
/// **Utilisé dans** : `database.rs::insert_event()`
pub const SQL_INSERT_EVENT: &str = r#"
    INSERT INTO events_today (
        username, action, timestamp, hostname, source_ip, server_timestamp,
        os_name, os_version, kernel_version, hardware_info, session_uuid
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
"#;

// ============================================================================
// REQUÊTES D'ANALYSE (utilisées dans la documentation README.md)
// ============================================================================
//
// Note : Les requêtes ci-dessous ne sont PAS utilisées dans le code Rust actuellement.
//        Elles sont documentées ici pour référence et usage futur potentiel.
//        Elles sont actuellement mentionnées dans README.md pour analyse manuelle.

#[allow(dead_code)]  // Usage futur ou requêtes manuelles SQL
/// Liste toutes les sessions actuellement ouvertes (connexion sans déconnexion associée).
/// 
/// **Objectif** : Monitoring en temps réel - identifier les utilisateurs actuellement connectés.
/// 
/// **Logique** :
/// - Recherche toutes les connexions (action='C') dans events_today
/// - Exclut celles qui ont une déconnexion (action='D') associée via session_uuid
/// - Trie par hostname puis timestamp (plus anciennes en premier par machine)
/// 
/// **Paramètres** : Aucun
/// 
/// **Colonnes retournées** :
/// - `username` : Nom d'utilisateur
/// - `hostname` : Nom de la machine
/// - `connected_at` : Date/heure de connexion (alias de timestamp)
/// - `session_uuid` : Identifiant de session
/// - `source_ip` : Adresse IP source
/// - `os_name` : Nom du système d'exploitation
/// - `os_version` : Version du système d'exploitation
/// 
/// **Utilisé dans** : `handlers.rs::get_current_sessions()` (endpoint GET /api/v1/sessions/current)
pub const SQL_LIST_OPEN_SESSIONS: &str = r#"
    SELECT 
        username,
        hostname,
        timestamp AS connected_at,
        session_uuid,
        source_ip,
        os_name,
        os_version
    FROM events_today
    WHERE action = 'C'
      AND NOT EXISTS (
          SELECT 1 FROM events_today e2
          WHERE e2.session_uuid = events_today.session_uuid
            AND e2.action = 'D'
      )
    ORDER BY hostname ASC, timestamp ASC
"#;

#[allow(dead_code)]  // Usage futur ou requêtes manuelles SQL
/// Calcule la durée des sessions terminées (avec connexion ET déconnexion).
/// 
/// **Objectif** : Analyse des temps de session - statistiques d'utilisation.
/// 
/// **Logique** :
/// - Joint les connexions (c.action='C') avec leurs déconnexions (d.action='D')
/// - Calcule la durée en minutes : (julianday(disconnect) - julianday(connect)) * 1440
/// - Limite aux 50 sessions les plus récentes par défaut
/// 
/// **Paramètres** : Aucun (mais LIMIT peut être ajusté)
/// 
/// **Colonnes retournées** :
/// - `username` : Nom d'utilisateur
/// - `hostname` : Nom de la machine
/// - `connected_at` : Timestamp de connexion
/// - `disconnected_at` : Timestamp de déconnexion
/// - `duration_minutes` : Durée de la session en minutes (arrondi)
/// - `session_uuid` : Identifiant de session
/// 
/// **Usage** : Requête manuelle ou future API d'analyse
pub const SQL_SESSION_DURATIONS: &str = r#"
    SELECT 
        c.username,
        c.hostname,
        c.timestamp AS connected_at,
        d.timestamp AS disconnected_at,
        ROUND((julianday(d.timestamp) - julianday(c.timestamp)) * 1440, 2) AS duration_minutes,
        c.session_uuid
    FROM events_today c
    INNER JOIN events_today d 
        ON c.session_uuid = d.session_uuid 
       AND d.action = 'D'
    WHERE c.action = 'C'
    ORDER BY c.timestamp DESC
    LIMIT 50
"#;

#[allow(dead_code)]  // Usage futur ou requêtes manuelles SQL
/// Compte le nombre de sessions par utilisateur (classement des plus actifs).
/// 
/// **Objectif** : Statistiques d'usage - identifier les utilisateurs les plus actifs.
/// 
/// **Logique** :
/// - Compte toutes les connexions (action='C') par username
/// - Trie par nombre de sessions décroissant (plus actifs en premier)
/// - Limite aux 20 premiers par défaut
/// 
/// **Paramètres** : Aucun (mais LIMIT peut être ajusté)
/// 
/// **Colonnes retournées** :
/// - `username` : Nom d'utilisateur
/// - `session_count` : Nombre total de sessions
/// - `last_connection` : Timestamp de la dernière connexion
/// 
/// **Usage** : Requête manuelle ou future API de statistiques
pub const SQL_TOP_USERS_BY_SESSION_COUNT: &str = r#"
    SELECT 
        username,
        COUNT(*) AS session_count,
        MAX(timestamp) AS last_connection
    FROM events_today
    WHERE action = 'C'
    GROUP BY username
    ORDER BY session_count DESC
    LIMIT 20
"#;
