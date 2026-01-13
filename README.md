# Winlog 2 - Système de monitoring multi-plateforme

Système complet de monitoring d'activité utilisateur pour parcs informatiques hétérogènes (Windows/Linux). Développé en Rust pour le client, avec un serveur PHP (migration Rust prévue).

## 🎯 Objectif

Monitorer en temps réel les connexions/déconnexions et inventorier le matériel d'un parc de machines via des clients légers déployés sur chaque poste, centralisant les données sur un serveur pour analyse SQL et reporting.

## 🏗️ Architecture globale

Le projet est divisé en **2 parties indépendantes** :

### 📦 Structure du repository

```
winlog2/
├── client/              # Client Rust multi-plateforme (Windows + Linux)
│   ├── src/
│   │   ├── bin/        # 3 binaires : logon, logout, matos
│   │   ├── config.rs   # Configuration centralisée
│   │   └── lib.rs      # Librairie partagée
│   ├── Cargo.toml
│   └── README.md       # Documentation client
│
├── serveur/            # Serveur de collecte et stockage
│   ├── php/           # Implémentation PHP actuelle
│   │   ├── index.php  # Point d'entrée HTTP POST
│   │   ├── config.php # Configuration serveur
│   │   └── *.php      # Scripts de gestion DB
│   └── README.md      # Documentation serveur
│
├── README.md          # Cette documentation globale
└── .github/
    └── copilot-instructions.md  # Instructions développement
```

## 🖥️ Partie Client (Rust)

### 3 Binaires multi-plateformes

#### `logon` / `logon.exe`
- **Plateformes** : Windows 10/11, Linux (Ubuntu, Debian, RHEL, Arch...)
- **Déclencheur** : Script d'ouverture de session (GPO Windows / PAM Linux)
- **Action** : Code "C" (Connexion)
- **Données** : Username, timestamp, hostname, OS, architecture
- **Performance** : <100ms d'exécution

#### `logout` / `logout.exe`
- **Plateformes** : Windows 10/11, Linux
- **Déclencheur** : Script de fermeture de session
- **Action** : Code "D" (Déconnexion)
- **Données** : Username, timestamp, durée de session
- **Performance** : <100ms d'exécution

#### `matos` / `matos.exe`
- **Plateformes** : Windows 10/11, Linux
- **Déclencheur** : Tâche planifiée ou exécution manuelle
- **Action** : Code "M" (Matériel)
- **Données** : CPU, RAM, disques, réseau, périphériques
- **Performance** : <500ms d'exécution (collecte détaillée)

### Caractéristiques techniques

**Architecture 100% synchrone** :
- Démarrage instantané (~10ms)
- Empreinte mémoire minimale (<5MB)
- Binaires légers (~1MB chaque)
- Pas de runtime async (optimisé one-shot)

**Stack Rust** :
- `sysinfo` : Collecte système multi-plateforme
- `minreq` : Client HTTP léger sans dépendances lourdes
- `serde` + `serde_json` : Sérialisation JSON
- `chrono` : Timestamps ISO 8601 UTC
- `whoami` : Détection username Windows/Linux

**Communication** :
- HTTP POST avec payload JSON
- Retry automatique (3 tentatives)
- Timeout configurable (défaut 30s)
- Support HTTPS natif

**Compilation cross-platform** :
- Windows : MinGW (GCC) ou MSVC
- Linux : GCC/rustc natif
- Targets : `x86_64-pc-windows-gnu`, `x86_64-unknown-linux-gnu`

### Configuration client

Modifier `client/src/config.rs` :
```rust
pub const SERVER_URL: &str = "http://monitoring.local/winlog/index.php";
pub const HTTP_TIMEOUT_SECS: u64 = 30;
pub const MAX_RETRIES: u32 = 3;
pub const RETRY_DELAY_MS: u64 = 500;
```

## 🌐 Partie Serveur

### Implémentation actuelle : PHP + SQLite

**Point d'entrée** : `serveur/php/index.php`
- Réception HTTP POST
- Validation User-Agent et JSON
- Stockage en base SQLite avec gestion intelligente des sessions
- Réponse JSON avec statut et event_id

**Base de données** : SQLite en mode WAL
- Table `events` avec 13 colonnes
- 6 index pour requêtes optimisées
- Support concurrence (lectures pendant écritures)
- Transaction ACID

**Scripts de gestion** :
- `creation_base.php` : Initialisation DB
- `purge_base.php` : Vidage données (conserve structure)
- `delete_base.php` : Suppression complète

### Migration future : Rust + Framework web

**Prévu** :
- Framework : Actix-web ou Axum
- ORM : SQLx (requêtes type-safe)
- Avantages : 5-10x plus performant, binaire unique, cohérence client/serveur

## 📊 Format des données échangées

```json
{
  "username": "jerome",
  "action": "C",
  "timestamp": "2026-01-13T08:30:00Z",
  "hostname": "WORKSTATION-01",
  "os_info": {
    "os_name": "Windows",
    "os_version": "11 (26200)",
    "kernel_version": "10.0.22631"
  },
  "hardware_info": {
    "cpu_count": 12,
    "cpu_brand": "Intel Core i7-12700K",
    "memory_total": 33554432
  }
}
```

**Codes d'action** :
- `"C"` : Connexion (ouverture de session)
- `"D"` : Déconnexion (fermeture de session)
- `"M"` : Matériel (inventaire hardware détaillé)

**Optimisations** :
- Codes courts pour réduire la bande passante (~500 octets/événement)
- Structure cohérente entre tous les types d'événements
- Timestamps UTC ISO 8601 pour compatibilité internationale

## 🗄️ Base de données SQLite

### Emplacement
- **Fichier** : `/var/lib/winlog/winlog.db` (configurable)
- **Mode** : WAL (Write-Ahead Logging)
- **Permissions** : 644, owner `www-data` (ou utilisateur serveur web)

### Table `events`

| Colonne | Type | Description |
|---------|------|-------------|
| `id` | INTEGER PK | Identifiant unique auto-incrémenté |
| `username` | VARCHAR(50) | Nom d'utilisateur (Windows/Linux) |
| `action` | CHAR(1) | 'C', 'D', ou 'M' |
| `timestamp` | DATETIME | Timestamp client (ISO 8601) |
| `hostname` | VARCHAR(100) | Nom de la machine |
| `source_ip` | VARCHAR(45) | IP source (IPv4/IPv6) |
| `server_timestamp` | DATETIME | Timestamp réception serveur |
| `os_name` | VARCHAR(50) | Nom OS |
| `os_version` | VARCHAR(100) | Version OS |
| `kernel_version` | VARCHAR(50) | Version noyau |
| `hardware_info` | TEXT | JSON matériel (action='M') |
| `session_uuid` | VARCHAR(100) | Identifiant session unique |
| `created_at` | DATETIME | Timestamp insertion DB |

### Index optimisés
- `idx_username_action` : Requêtes par utilisateur/action
- `idx_timestamp` : Tri chronologique
- `idx_hostname` : Filtrage par machine
- `idx_action_timestamp` : Évolution temporelle
- `idx_session_uuid` : Requêtes par session
- `idx_source_ip` : Filtrage par IP

## 🚀 Déploiement

### Client - Windows (GPO)

1. **Compiler les binaires Windows** :
```bash
# Sur Linux (cross-compilation)
rustup target add x86_64-pc-windows-gnu
cd client && cargo build --release --target x86_64-pc-windows-gnu

# Sur Windows (natif)
cd client && cargo build --release
```

2. **Copier vers SYSVOL** :
```cmd
copy target\release\logon.exe \\DOMAIN\SYSVOL\scripts\
copy target\release\logout.exe \\DOMAIN\SYSVOL\scripts\
copy target\release\matos.exe \\DOMAIN\SYSVOL\scripts\
```

3. **Configurer GPO** :
   - **Ouverture** : `User Configuration > Scripts > Logon > Add logon.exe`
   - **Fermeture** : `User Configuration > Scripts > Logoff > Add logout.exe`
   - **Inventaire** : Tâche planifiée quotidienne pour `matos.exe`

### Client - Linux (PAM/Systemd)

1. **Compiler les binaires Linux** :
```bash
cd client && cargo build --release
```

2. **Installer les binaires** :
```bash
sudo cp target/release/{logon,logout,matos} /usr/local/bin/
sudo chmod 755 /usr/local/bin/{logon,logout,matos}
```

3. **Configurer PAM** :
```bash
# Ouverture : /etc/profile.d/winlog-logon.sh
#!/bin/bash
/usr/local/bin/logon &

# Fermeture : /etc/bash.bash_logout ou ~/.bash_logout
/usr/local/bin/logout &
```

4. **Tâche cron pour inventaire** :
```bash
sudo crontab -e
# Ajouter : 0 2 * * * /usr/local/bin/matos
```

### Serveur - PHP + SQLite

1. **Prérequis** :
```bash
sudo apt install php php-sqlite3 apache2
```

2. **Déployer les fichiers** :
```bash
sudo cp -r serveur/php /var/www/html/winlog
```

3. **Créer la base de données** :
```bash
cd /var/www/html/winlog
php creation_base.php
```

4. **Configurer les permissions** :
```bash
sudo mkdir -p /var/lib/winlog
sudo chown www-data:www-data /var/lib/winlog
sudo chmod 755 /var/lib/winlog
```

5. **Configurer Apache/Nginx** :
   - Activer `mod_rewrite` et `mod_headers`
   - Configurer HTTPS (Let's Encrypt recommandé)
   - Limiter accès réseau (firewall)

## 🧪 Tests et validation

### Test client local
```bash
# Modifier temporairement SERVER_URL dans client/src/config.rs
# puis compiler et tester
cd client
cargo build --release
./target/release/logon
./target/release/logout
./target/release/matos
```

### Test serveur
```bash
curl -X POST http://localhost/winlog/index.php \
  -H "Content-Type: application/json" \
  -H "User-Agent: Winlog/0.1.0 (Windows)" \
  -d '{
    "username": "test",
    "action": "C",
    "timestamp": "2026-01-13T08:30:00Z",
    "hostname": "TEST-PC",
    "os_info": {"os_name": "Windows", "os_version": "11", "kernel_version": "10.0.22631"}
  }'
```

### Vérifier la base de données
```bash
sqlite3 /var/lib/winlog/winlog.db \
  "SELECT username, action, timestamp FROM events ORDER BY id DESC LIMIT 10;"
```

## 🔍 Requêtes SQL d'analyse

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

### Durée des sessions terminées (dernières 50)
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
    COUNT(CASE WHEN action='M' THEN 1 END) as inventaires
FROM events 
GROUP BY DATE(timestamp)
ORDER BY jour DESC;
```

### Top 20 utilisateurs actifs
```sql
SELECT 
    username, 
    COUNT(*) as total_connexions,
    MAX(timestamp) as derniere_activite
FROM events 
WHERE action='C'
GROUP BY username
ORDER BY total_connexions DESC
LIMIT 20;
```

## 📖 Documentation détaillée

- **Client Rust** : `/client/README.md` - Compilation, configuration, déploiement Windows/Linux
- **Serveur PHP** : `/serveur/README.md` - Installation, gestion DB, migration Rust
- **Scripts PHP** : `/serveur/php/README.md` - Documentation technique détaillée
- **Instructions dev** : `/.github/copilot-instructions.md` - Guide développement

## 🛠️ Développement

### Structure complète du projet
```
winlog2/
├── client/
│   ├── src/
│   │   ├── bin/
│   │   │   ├── logon.rs     # Binaire connexion
│   │   │   ├── logout.rs    # Binaire déconnexion
│   │   │   └── matos.rs     # Binaire inventaire
│   │   ├── config.rs        # Configuration client
│   │   └── lib.rs           # Modules partagés
│   ├── Cargo.toml           # Dépendances Rust
│   └── README.md
│
├── serveur/
│   ├── php/
│   │   ├── config.php       # Configuration serveur
│   │   ├── index.php        # Endpoint HTTP
│   │   ├── index_sql.php    # Requêtes SQL
│   │   ├── creation_base.php
│   │   ├── purge_base.php
│   │   ├── delete_base.php
│   │   └── README.md
│   └── README.md
│
├── README.md                # Documentation globale
└── .github/
    └── copilot-instructions.md
```

### Workflow de développement
1. **Modification client** : Éditer `client/src/*.rs`
2. **Vérifier compilation** : `cd client && cargo check`
3. **Build release** : `cargo build --release`
4. **Tester** : Exécuter binaires sur Windows/Linux
5. **Mettre à jour docs** : README.md concernés

### Ajout de fonctionnalités
- **Client** : Modifier `client/src/lib.rs` (modules)
- **Serveur PHP** : Modifier `serveur/php/index.php`
- **Base de données** : Modifier `serveur/php/creation_base.php` (schéma)

## 🔐 Sécurité

### Client
- Pas de données sensibles dans le code
- Support HTTPS via certificats système
- User-Agent custom pour identification
- Pas d'exécution de commandes shell

### Serveur
- Validation stricte User-Agent et JSON
- Transactions ACID (pas de corruption)
- Firewall réseau recommandé
- HTTPS obligatoire en production
- Rate limiting (nginx `limit_req`)

### Recommandations production
- **HTTPS** : Certificat Let's Encrypt
- **Firewall** : Limiter au réseau interne uniquement
- **Backups** : Sauvegarde quotidienne de `/var/lib/winlog/winlog.db`
- **Monitoring** : Surveiller logs Apache/Nginx et taille DB
- **Rotation** : Archiver/purger anciennes données (>6 mois)

## 📊 Performances

### Client
- **Démarrage** : ~10ms
- **Exécution** : <100ms (logon/logout), <500ms (matos)
- **Mémoire** : <5MB
- **Binaires** : ~800KB-1.2MB après strip
- **Réseau** : ~500 octets par événement

### Serveur
- **Concurrence** : Centaines de connexions simultanées (mode WAL)
- **Latence** : <50ms par requête (réseau local)
- **Stockage** : ~200 octets par événement en DB
- **Index** : Requêtes complexes <10ms

## 🗺️ Roadmap

### Phase actuelle : Stabilisation multi-plateforme ✅
- [x] Client Rust fonctionnel Windows
- [x] Serveur PHP + SQLite opérationnel
- [x] Réorganisation repository (client/serveur)
- [ ] Tests approfondis Linux (Ubuntu, Debian, RHEL)
- [ ] Documentation déploiement PAM Linux
- [ ] Scripts d'installation automatisée

### Phase 2 : Migration serveur Rust 🔜
- [ ] POC Actix-web + SQLx
- [ ] Migration endpoints HTTP
- [ ] Tests de charge (1000+ clients)
- [ ] Packaging serveur (binaire unique)

### Phase 3 : Fonctionnalités avancées 🚀
- [ ] Authentification clients (tokens/certificats)
- [ ] Dashboard web temps réel
- [ ] Alertes (sessions anormales, nouveaux matériels)
- [ ] Export rapports (PDF, Excel)
- [ ] API REST pour intégrations tierces

## 🤝 Contribution

### Standards de code
- **Rust** : `rustfmt` et `clippy` obligatoires
- **PHP** : PSR-12 coding standard
- **Commits** : Messages descriptifs en français
- **Documentation** : Mise à jour README.md synchrone avec le code

### Tests
- **Client** : `cargo test` et compilation multi-plateforme
- **Serveur** : Tests manuels avec curl et vérification DB

## 📜 Licence

Projet interne - Usage restreint

## 📧 Contact

Maintainer : Jerome
Repository : `/home/jerome/scripts/rust/winlog2`

---

**Version** : 0.1.0  
**Dernière mise à jour** : 13 janvier 2026  
**Statut** : Phase de stabilisation multi-plateforme
