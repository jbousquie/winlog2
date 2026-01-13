# Documentation Serveur Winlog - PHP

## Vue d'ensemble

Le serveur Winlog traite les données de monitoring Windows envoyées par les clients Rust via HTTP POST. Il utilise une base de données SQLite pour stocker et analyser les événements de connexion, déconnexion et informations matérielles.

## Architecture du serveur

### 📁 Structure des fichiers

- **`config.php`** : Configuration commune à tous les scripts (chemins, constantes)
- **`index.php`** : Point d'entrée principal qui reçoit et traite les requêtes HTTP POST
- **`index_sql.php`** : Définitions des requêtes SQL utilisées par `index.php`
- **`creation_base.php`** : Script d'initialisation de la base de données SQLite
- **`delete_base.php`** : Script de suppression complète de la base de données
- **`purge_base.php`** : Script de vidage des données (conserve la structure)

## 🔧 Scripts de gestion

### `config.php` - Configuration centralisée
**Fonction** : Centralise tous les paramètres de configuration pour faciliter la maintenance

**Constantes définies** :
- **`DB_PATH`** : Chemin complet vers la base SQLite (`/home/jerome/winlog/winlog.db`)
- **`DB_DIR`** : Répertoire de stockage (`/home/jerome/winlog`)
- **`EXPECTED_USER_AGENT`** : User-Agent attendu des clients (`Winlog/0.1.0 (Windows)`)
- **`VALID_ACTIONS`** : Codes d'action autorisés (`['C', 'D', 'M']`)
- **`SQLITE_PRAGMA_CONFIG`** : Configuration SQLite optimisée (WAL, cache, timeouts)

**Utilisation** : `require_once 'config.php';` dans tous les scripts PHP

### `index.php` - Serveur principal
**Fonction** : Récepteur HTTP POST pour les données des clients Winlog

**Fonctionnalités** :
- **Validation stricte** : Méthode POST, User-Agent `Winlog/0.1.0 (Windows)`, Content-Type JSON
- **Parsing JSON** : Décodage et validation de la structure des données reçues
- **Gestion des sessions** : 
  - Connexion (C) : Ferme automatiquement les sessions ouvertes du jour avant d'en créer une nouvelle
  - Déconnexion (D) : Associe à la dernière session ouverte
  - Matériel (M) : Enregistre les informations hardware
- **Détection IP** : Identification de l'IP source réelle (support proxies, Cloudflare)
- **Stockage SQLite** : Transaction ACID pour garantir l'intégrité
- **Logging** : Erreurs et succès dans les logs système PHP

**Réponse JSON** :
```json
{
  "status": "success",
  "message": "Data stored in database",
  "event_id": 123,
  "session_uuid": "jerome@pc01@abc123",
  "action": "C",
  "username": "jerome"
}
```

### `index_sql.php` - Requêtes SQL centralisées
**Fonction** : Centralisation de toutes les requêtes SQL pour maintenance facilitée

**Constantes définies** :
- `SQL_FIND_LAST_OPEN_SESSION` : Trouve la dernière session ouverte (pour déconnexions)
- `SQL_FIND_OPEN_SESSION_TODAY` : Cherche session ouverte le même jour (pour fermeture auto)
- `SQL_INSERT_EVENT` : Insertion d'un nouvel événement
- `SQL_INSERT_AUTO_DISCONNECT` : Insertion d'une déconnexion automatique
- Requêtes utilitaires pour statistiques et administration

### `creation_base.php` - Initialisation
**Fonction** : Création de la base de données SQLite et de sa structure

**Actions** :
1. Vérification/création du répertoire `/home/jerome/winlog/`
2. Création du fichier `winlog.db`
3. Configuration optimale SQLite (mode WAL, cache, timeouts)
4. Création de la table `events` avec contraintes
5. Création de 6 index pour optimiser les performances
6. Affichage des informations de création

**Usage** : `php creation_base.php`

### `delete_base.php` - Suppression complète
**Fonction** : Suppression définitive de la base de données

**Sécurités** :
- Confirmation obligatoire "SUPPRIMER"
- Affichage du nombre d'enregistrements avant suppression
- Suppression des fichiers WAL et SHM associés

**Usage** : `php delete_base.php`

### `purge_base.php` - Vidage des données
**Fonction** : Suppression de toutes les données en conservant la structure

**Actions** :
- Statistiques détaillées avant vidage (total, répartition par action)
- Confirmation obligatoire "VIDER"
- Transaction sécurisée avec rollback
- Réinitialisation de l'auto-increment
- VACUUM automatique pour récupérer l'espace disque

**Usage** : `php purge_base.php`

## 🗄️ Structure de la base SQLite

### Emplacement
- **Fichier** : `/home/jerome/winlog/winlog.db`
- **Mode** : WAL (Write-Ahead Logging) pour concurrence optimale
- **Permissions** : 644 ou 755 selon configuration serveur

### Table `events`

| Champ | Type | Description |
|-------|------|-------------|
| `id` | INTEGER PK AUTOINCREMENT | Identifiant unique, tri chronologique |
| `username` | VARCHAR(50) NOT NULL | Nom d'utilisateur Windows |
| `action` | CHAR(1) NOT NULL | Code d'action : 'C'=Connexion, 'D'=Déconnexion, 'M'=Matériel |
| `timestamp` | DATETIME NOT NULL | Timestamp client (ISO 8601) |
| `hostname` | VARCHAR(100) | Nom de la machine Windows |
| `source_ip` | VARCHAR(45) | Adresse IP source de la requête |
| `server_timestamp` | DATETIME | Timestamp serveur à la réception |
| `os_name` | VARCHAR(50) | Nom de l'OS (ex: "Windows") |
| `os_version` | VARCHAR(100) | Version OS (ex: "11 (26200)") |
| `kernel_version` | VARCHAR(50) | Version noyau |
| `hardware_info` | TEXT | Informations matérielles (JSON, action='M' uniquement) |
| `session_uuid` | VARCHAR(100) | Identifiant de session (format: user@host@hash6) |
| `created_at` | DATETIME | Timestamp de création en base |

### Index de performance

```sql
CREATE INDEX idx_username_action ON events(username, action);     -- Sessions par utilisateur
CREATE INDEX idx_timestamp ON events(timestamp);                  -- Tri chronologique
CREATE INDEX idx_hostname ON events(hostname);                    -- Filtrage par machine
CREATE INDEX idx_action_timestamp ON events(action, timestamp);   -- Actions dans le temps
CREATE INDEX idx_session_uuid ON events(session_uuid);           -- Requêtes par session
CREATE INDEX idx_source_ip ON events(source_ip);                 -- Filtrage par IP
```

## 🔄 Logique de gestion des sessions

### Connexion (action='C')
1. **Vérification** : Recherche d'une session ouverte pour le même utilisateur/machine/jour
2. **Fermeture automatique** : Si trouvée, insertion d'une déconnexion automatique (timestamp - 1 seconde)
3. **Nouvelle session** : Génération d'un `session_uuid` unique et insertion de la connexion

### Déconnexion (action='D')
1. **Recherche** : Trouve la dernière session ouverte pour cet utilisateur/machine
2. **Association** : Utilise le même `session_uuid` que la connexion correspondante
3. **Gestion des orphelines** : Si aucune session ouverte trouvée, crée un UUID `orphan_*`

### Matériel (action='M')
1. **UUID spécial** : Génère un identifiant `hardware_*` 
2. **Données étendues** : Stocke les informations matérielles complètes en JSON

## 📊 Requêtes d'analyse utiles

### Sessions actuellement ouvertes
```sql
SELECT username, hostname, session_uuid, timestamp, source_ip
FROM events 
WHERE action='C' 
AND NOT EXISTS (
    SELECT 1 FROM events e2 
    WHERE e2.session_uuid = events.session_uuid 
    AND e2.action = 'D'
)
ORDER BY timestamp DESC;
```

### Durée des sessions terminées
```sql
SELECT 
    c.username, c.hostname,
    c.timestamp as connexion,
    d.timestamp as deconnexion,
    (julianday(d.timestamp) - julianday(c.timestamp)) * 24 * 60 as duree_minutes
FROM events c
JOIN events d ON c.session_uuid = d.session_uuid
WHERE c.action='C' AND d.action='D'
ORDER BY c.timestamp DESC;
```

### Statistiques par jour
```sql
SELECT 
    DATE(timestamp) as jour,
    COUNT(CASE WHEN action='C' THEN 1 END) as connexions,
    COUNT(CASE WHEN action='D' THEN 1 END) as deconnexions,
    COUNT(CASE WHEN action='M' THEN 1 END) as materiels
FROM events 
GROUP BY DATE(timestamp)
ORDER BY jour DESC;
```

## ⚙️ Configuration et déploiement

### Prérequis serveur
- **PHP** : Version 7.4+ (recommandé 8.0+)
- **Extensions** : PDO, SQLite3
- **Permissions** : Écriture sur `/home/jerome/winlog/`
- **Apache/Nginx** : Configuration pour recevoir POST JSON

### Installation
1. Copier tous les fichiers PHP dans le répertoire web (`/var/www/html/winlog/`)
2. Exécuter `php creation_base.php` pour initialiser la base
3. Vérifier les permissions du répertoire de base de données
4. Tester avec une requête POST depuis un client

### Maintenance
- **Logs** : Surveiller `/var/log/apache2/error.log` pour les erreurs Winlog
- **Espace disque** : La base grandit avec le temps, prévoir rotation/archivage
- **Performance** : Mode WAL permet lectures pendant écritures
- **Sauvegarde** : Sauvegarder régulièrement `/home/jerome/winlog/winlog.db`
- **Configuration** : Modifier `config.php` pour changer les paramètres globaux

### Dépannage
- **Erreur 500** : Vérifier permissions répertoire et fichier base
- **Timeout** : Augmenter `busy_timeout` SQLite si forte concurrence
- **Corruption** : Utiliser `.integrity_check` SQLite pour vérifier
- **Espace** : Utiliser `VACUUM` pour optimiser l'espace (automatique dans purge_base.php)