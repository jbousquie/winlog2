# Winlog Serveur - Collecteur de données de monitoring

Serveur de collecte et stockage centralisé pour les événements de monitoring Winlog. Reçoit les données des clients via HTTP POST et les stocke dans une base SQLite pour analyse en temps réel.

## 🎯 Objectif

Centraliser et persister les événements de connexion/déconnexion et informations matérielles provenant d'un parc de machines Windows/Linux, avec requêtage SQL performant.

## 🏗️ Architecture actuelle (PHP)

Le serveur est actuellement implémenté en **PHP** avec SQLite. Une migration vers **Rust** (framework web léger) est prévue pour améliorer les performances et la cohérence avec le client.

### 📁 Structure des fichiers

```
serveur/
└── php/
    ├── config.php           # Configuration centralisée
    ├── index.php            # Point d'entrée HTTP POST
    ├── index_sql.php        # Requêtes SQL centralisées
    ├── creation_base.php    # Script d'initialisation DB
    ├── delete_base.php      # Script de suppression DB
    ├── purge_base.php       # Script de vidage données
    └── README.md            # Documentation détaillée PHP
```

## 🚀 Fonctionnalités

### Réception HTTP POST
- **Endpoint** : `/index.php`
- **Méthode** : POST uniquement
- **Content-Type** : `application/json`
- **Validation** : User-Agent `Winlog/0.1.0`, structure JSON, codes d'action

### Stockage SQLite
- **Base** : `/var/lib/winlog/winlog.db` (configurable)
- **Mode** : WAL (Write-Ahead Logging) pour concurrence
- **Table** : `events` avec 6 index optimisés
- **Transactions** : ACID pour garantir l'intégrité

### Gestion intelligente des sessions
- **Connexion (C)** : Ferme automatiquement les sessions ouvertes du même jour avant d'en créer une nouvelle
- **Déconnexion (D)** : Associe à la dernière session ouverte (via `session_uuid`)
- **Matériel (M)** : Stocke les informations hardware en JSON

### Détection IP source
- Support proxies et CDN (Cloudflare, X-Forwarded-For)
- Journalisation de l'IP réelle du client

## 🗄️ Base de données SQLite

### Table `events`

| Colonne | Type | Description |
|---------|------|-------------|
| `id` | INTEGER PK | Identifiant unique auto-incrémenté |
| `username` | VARCHAR(50) | Nom d'utilisateur (Windows/Linux) |
| `action` | CHAR(1) | Code action : 'C', 'D', 'M' |
| `timestamp` | DATETIME | Timestamp client (ISO 8601 UTC) |
| `hostname` | VARCHAR(100) | Nom de la machine |
| `source_ip` | VARCHAR(45) | Adresse IP source |
| `server_timestamp` | DATETIME | Timestamp de réception serveur |
| `os_name` | VARCHAR(50) | Système d'exploitation |
| `os_version` | VARCHAR(100) | Version OS |
| `kernel_version` | VARCHAR(50) | Version noyau |
| `hardware_info` | TEXT | JSON matériel (action='M') |
| `session_uuid` | VARCHAR(100) | Identifiant de session unique |
| `created_at` | DATETIME | Timestamp d'insertion DB |

### Index de performance

```sql
idx_username_action      -- Requêtes par utilisateur/action
idx_timestamp            -- Tri chronologique
idx_hostname             -- Filtrage par machine
idx_action_timestamp     -- Évolution temporelle par action
idx_session_uuid         -- Requêtes par session
idx_source_ip            -- Filtrage par IP
```

## 🔧 Installation (PHP actuel)

### Prérequis
- **PHP** : 7.4+ (recommandé 8.0+)
- **Extensions** : PDO, SQLite3
- **Serveur web** : Apache ou Nginx
- **Permissions** : Écriture sur répertoire base de données

### Configuration

1. **Modifier `config.php`** :
```php
define('DB_PATH', '/var/lib/winlog/winlog.db');
define('DB_DIR', '/var/lib/winlog');
define('EXPECTED_USER_AGENT', 'Winlog/0.1.0 (Windows)');
```

2. **Créer la base de données** :
```bash
php creation_base.php
```

3. **Configurer les permissions** :
```bash
sudo mkdir -p /var/lib/winlog
sudo chown www-data:www-data /var/lib/winlog
sudo chmod 755 /var/lib/winlog
```

4. **Déployer sur serveur web** :
```bash
cp -r php/* /var/www/html/winlog/
```

### Test de réception
```bash
curl -X POST http://localhost/winlog/index.php \
  -H "Content-Type: application/json" \
  -H "User-Agent: Winlog/0.1.0 (Windows)" \
  -d '{
    "username": "test",
    "action": "C",
    "timestamp": "2026-01-13T08:30:00Z",
    "hostname": "TEST-PC",
    "os_info": {
      "os_name": "Windows",
      "os_version": "11",
      "kernel_version": "10.0.22631"
    }
  }'
```

## 📊 Scripts de gestion

### `creation_base.php` - Initialisation
Crée la base SQLite, la table `events` et les 6 index.

```bash
php creation_base.php
```

### `purge_base.php` - Vidage données
Supprime toutes les données en conservant la structure. Affiche des statistiques avant vidage.

```bash
php purge_base.php
# Confirmation : taper "VIDER"
```

### `delete_base.php` - Suppression complète
Supprime définitivement la base de données.

```bash
php delete_base.php
# Confirmation : taper "SUPPRIMER"
```

## 📈 Requêtes SQL utiles

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
ORDER BY c.timestamp DESC
LIMIT 50;
```

### Statistiques quotidiennes
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

### Top utilisateurs actifs
```sql
SELECT 
    username, 
    COUNT(*) as total_connexions,
    MAX(timestamp) as derniere_connexion
FROM events 
WHERE action='C'
GROUP BY username
ORDER BY total_connexions DESC
LIMIT 20;
```

## 🔄 Migration future vers Rust

### Framework envisagé
- **Actix-web** ou **Axum** : Performances élevées, async natif
- **SQLx** : Requêtes SQL type-safe avec support SQLite
- **Tokio** : Runtime async pour gérer la concurrence
- **Serde** : Parsing JSON (déjà utilisé côté client)

### Avantages attendus
- **Performances** : 5-10x plus rapide que PHP
- **Concurrence** : Gestion native de milliers de connexions simultanées
- **Type-safety** : Détection d'erreurs à la compilation
- **Cohérence** : Même langage client/serveur
- **Binaire unique** : Déploiement simplifié sans dépendances PHP

### Structure cible
```
serveur/
├── src/
│   ├── main.rs          # Point d'entrée web
│   ├── routes.rs        # Endpoints HTTP
│   ├── models.rs        # Structures DB
│   ├── database.rs      # Connexion SQLite
│   └── config.rs        # Configuration
├── Cargo.toml
└── README.md
```

## 🛠️ Maintenance

### Logs
```bash
# Logs Apache
tail -f /var/log/apache2/error.log | grep Winlog

# Logs Nginx
tail -f /var/log/nginx/error.log | grep Winlog
```

### Sauvegarde base de données
```bash
# Sauvegarde complète
sqlite3 /var/lib/winlog/winlog.db ".backup /backup/winlog-$(date +%Y%m%d).db"

# Export SQL
sqlite3 /var/lib/winlog/winlog.db .dump > /backup/winlog-$(date +%Y%m%d).sql
```

### Optimisation espace disque
```bash
# Compacter la base (déjà fait automatiquement dans purge_base.php)
sqlite3 /var/lib/winlog/winlog.db "VACUUM;"
```

### Monitoring
```bash
# Nombre total d'événements
sqlite3 /var/lib/winlog/winlog.db "SELECT COUNT(*) FROM events;"

# Taille de la base
du -h /var/lib/winlog/winlog.db

# Derniers événements
sqlite3 /var/lib/winlog/winlog.db \
  "SELECT username, action, timestamp FROM events ORDER BY id DESC LIMIT 10;"
```

## 🔐 Sécurité

### Protection actuelle
- Validation User-Agent côté serveur
- Vérification structure JSON stricte
- Transactions ACID (pas de corruption)
- Isolation réseau (firewall recommandé)

### Recommandations
- **HTTPS obligatoire** : Certificat Let's Encrypt
- **Firewall** : Limiter accès au réseau interne
- **Rate limiting** : Éviter flood (nginx `limit_req`)
- **Authentification future** : Tokens ou certificats clients

## 📖 Documentation détaillée

Pour plus d'informations sur l'implémentation PHP actuelle, consulter `/serveur/php/README.md`.

Pour la documentation globale du projet, voir `/README.md`.
