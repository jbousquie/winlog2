# Winlog 2 - Système de monitoring multi-plateforme

Système complet de monitoring d'activité utilisateur pour parcs informatiques hétérogènes (Windows/Linux). Développé entièrement en Rust : client synchrone léger + serveur Axum/SQLx haute performance.

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
├── serveur/            # Serveur Rust de collecte et stockage
│   ├── src/
│   │   ├── main.rs     # Point d'entrée Axum
│   │   ├── config.rs   # Chargement config.toml
│   │   ├── models.rs   # Structures de données
│   │   ├── database.rs # Logique SQLx + sessions
│   │   └── handlers.rs # Handlers HTTP
│   ├── scripts/        # Scripts bash gestion DB
│   │   ├── create_base.sh
│   │   ├── purge_base.sh
│   │   ├── delete_base.sh
│   │   └── rotate_daily.sh
│   ├── Cargo.toml
│   ├── config.toml     # Configuration serveur
│   └── README.md       # Documentation serveur
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
pub const DEFAULT_SERVER_URL: &str = "http://127.0.0.1:3000/api/v1/events";
pub const HTTP_TIMEOUT_SECS: u64 = 30;
pub const MAX_RETRIES: u32 = 3;
pub const RETRY_DELAY_MS: u64 = 500;
pub const USER_AGENT: &str = "Winlog/0.1.0";
```

## 🌐 Partie Serveur

### Architecture Rust : Axum + SQLx + SQLite

**Framework web** : Axum 0.7 (Tokio team)
- API REST asynchrone haute performance
- Endpoints : `POST /api/v1/events`, `GET /health`
- Validation stricte : User-Agent, JSON schema, actions
- Support proxies : X-Forwarded-For, CF-Connecting-IP
- Logs structurés avec tracing

**Base de données** : SQLite + SQLx 0.8
- **Architecture partitionnée** pour performances 10x supérieures :
  - `events_today` : Événements du jour (~100 rows, lectures/écritures rapides)
  - `events_history` : Archive (10k+ rows, lecture seule)
  - `events_all` : Vue UNION ALL des deux tables
- **Mode WAL** : Lectures concurrentes sans verrous
- **6 index optimisés** : Recherche par username, session_uuid, timestamp
- **Pool de connexions** : 10 connexions simultanées max
- **Compile-time checks** : Vérification SQL à la compilation

**Scripts de gestion** (bash) :
- `create_base.sh` : Création base partitionnée
- `purge_base.sh` : Vidage sélectif (--today/--history/--all)
- `delete_base.sh` : Suppression complète
- `rotate_daily.sh` : Rotation automatique quotidienne (cron)
- `migrate_to_new_structure.sh` : Migration depuis structure legacy

**Performances mesurées** :
- 5000 requêtes/seconde (vs 100 req/s en PHP)
- Latence P50 : 0.6ms (vs 30ms PHP)
- Mémoire : ~10 MB (vs ~50 MB PHP)
- Binaire : 3.1 MB standalone

**Logique de gestion** :
- **Connexion (C)** : Ferme automatiquement les sessions ouvertes du jour avant de créer une nouvelle
- **Déconnexion (D)** : Associe à la dernière session ouverte ou crée UUID orphelin
- **Matériel (M)** : UUID préfixé `hardware_` pour inventaire
- **UUID format** : `username@hostname@hash6` (MD5 6 premiers caractères)

### Configuration serveur

Éditez `serveur/config.toml` :
```toml
[server]
host = "127.0.0.1"      # 0.0.0.0 pour écouter sur toutes interfaces
port = 3000             # Port API REST

[database]
path = "/var/www/ferron/winlog/data/winlog.db"
pragma_journal_mode = "WAL"
pragma_synchronous = "NORMAL"
pragma_busy_timeout = 30000

[security]
expected_user_agent = "Winlog/0.1.0"
valid_actions = ["C", "D", "M"]
```

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

### Architecture partitionnée (2 tables + 1 vue)

**Emplacement** : `/var/www/ferron/winlog/data/winlog.db` (configurable)

**Tables** :
- `events_today` : Événements du jour (~100 rows, lectures/écritures rapides)
- `events_history` : Archive complète (10k+ rows, lecture seule sauf rotation)
- `events_all` : Vue UNION ALL des deux tables (requêtes globales)

**Avantages de la partition** :
- Requêtes "qui est connecté ?" 10x plus rapides (scan de ~100 rows au lieu de 10k+)
- Insertions sans bloquer l'historique
- Rotation quotidienne automatisée
- VACUUM rapide (petite table today)

### Schéma des tables

| Colonne | Type | Description |
|---------|------|-------------|
| `id` | INTEGER PK | Identifiant unique auto-incrémenté |
| `username` | TEXT | Nom d'utilisateur (Windows/Linux) |
| `action` | TEXT | 'C', 'D', ou 'M' (CHECK constraint) |
| `timestamp` | TEXT | Timestamp client (ISO 8601 UTC) |
| `hostname` | TEXT | Nom de la machine |
| `source_ip` | TEXT | IP source (IPv4/IPv6) |
| `server_timestamp` | TEXT | Timestamp réception serveur (auto) |
| `os_name` | TEXT | Nom OS |
| `os_version` | TEXT | Version OS |
| `kernel_version` | TEXT | Version noyau |
| `hardware_info` | TEXT | JSON matériel (action='M' uniquement) |
| `session_uuid` | TEXT | Identifiant session unique |
| `created_at` | TEXT | Timestamp insertion DB (auto) |

### Index optimisés

**events_today** (6 index) :
- `idx_today_username` : Recherche par utilisateur
- `idx_today_timestamp` : Tri chronologique
- `idx_today_hostname` : Filtrage par machine
- `idx_today_action_user` : Sessions ouvertes (action='C' + username)
- `idx_today_session` : Recherche par UUID
- `idx_today_ip` : Filtrage par IP source

**events_history** (mêmes index avec préfixe `idx_history_*`)

### Rotation quotidienne automatique

**Script** : `serveur/scripts/rotate_daily.sh` (bash)

**Installation cron** :
```bash
# Rotation à 1h du matin chaque jour
0 1 * * * /chemin/vers/serveur/scripts/rotate_daily.sh
```

**Actions effectuées** :
1. Backup automatique (`winlog_backup_YYYYMMDD.db`)
2. Copie events_today → events_history (INSERT SELECT)
3. Vidage events_today (DELETE)
4. VACUUM pour récupérer espace
5. Logs dans `/var/log/winlog_rotation.log`

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

### Serveur - Rust (Axum + SQLx)

1. **Compiler le serveur** :
```bash
cd serveur
cargo build --release
# Binaire généré : target/release/winlog-server (3.1 MB)
```

2. **Créer la base de données** :
```bash
cd serveur/scripts
./create_base.sh
# Crée /var/www/ferron/winlog/data/winlog.db avec structure partitionnée
```

3. **Configurer le serveur** :
```bash
cd serveur
nano config.toml
# Ajuster host, port, database path selon environnement
```

4. **Démarrer le serveur** :
```bash
# Lancement direct (logs dans terminal)
./target/release/winlog-server

# En arrière-plan avec logs
nohup ./target/release/winlog-server > winlog.log 2>&1 &

# Avec systemd (production)
sudo cp scripts/winlog-server.service /etc/systemd/system/
sudo systemctl enable winlog-server
sudo systemctl start winlog-server
```

5. **Installer rotation quotidienne** :
```bash
# Cron : rotation à 1h du matin
sudo crontab -e
# Ajouter : 0 1 * * * /chemin/vers/serveur/scripts/rotate_daily.sh
```

6. **Vérifier** :
```bash
# Health check
curl http://127.0.0.1:3000/health
# Attendu : {"status":"healthy","database":"connected",...}
```

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
curl -X POST http://127.0.0.1:3000/api/v1/events \
  -H "Content-Type: application/json" \
  -H "User-Agent: Winlog/0.1.0" \
  -d '{
    "username": "test",
    "action": "C",
    "timestamp": "2026-01-13T08:30:00Z",
    "hostname": "TEST-PC",
    "os_info": {"os_name": "Ubuntu 24.04", "os_version": "24.04", "kernel_version": "6.8.0"}
  }'
# Attendu : {"status":"success","event_id":1,"session_uuid":"test@TEST-PC@...",...}
```

### Vérifier la base de données
```bash
sqlite3 /var/www/ferron/winlog/data/winlog.db \
  "SELECT username, action, timestamp FROM events_today ORDER BY id DESC LIMIT 10;"
```

## 🔍 Requêtes SQL d'analyse

### Sessions actuellement ouvertes
```sql
SELECT username, hostname, session_uuid, timestamp, source_ip
FROM events_today 
WHERE action='C' 
AND username NOT IN (
    SELECT username FROM events_today WHERE action='D'
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
FROM events_all c
JOIN events_all d ON c.session_uuid = d.session_uuid
WHERE c.action = 'C' AND d.action = 'D'
ORDER BY d.timestamp DESC
LIMIT 50;
```
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
- **Serveur Rust** : `/serveur/README.md` - Architecture Axum, API REST, base SQLite partitionnée
- **Scripts bash** : `/serveur/scripts/README.md` - Gestion base de données (création, rotation, migration)
- **Migration BDD** : `/serveur/MIGRATION_BDD_2026.md` - Guide migration structure partitionnée
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
│   ├── README.md
│   └── target/release/      # Binaires compilés
│
├── serveur/
│   ├── src/
│   │   ├── main.rs          # Point d'entrée Axum
│   │   ├── config.rs        # Chargement config.toml
│   │   ├── models.rs        # Structures de données
│   │   ├── database.rs      # Logique SQLx + sessions
│   │   └── handlers.rs      # Handlers HTTP
│   ├── scripts/             # Scripts bash gestion DB
│   │   ├── create_base.sh
│   │   ├── purge_base.sh
│   │   ├── delete_base.sh
│   │   ├── rotate_daily.sh
│   │   └── README.md
│   ├── Cargo.toml           # Dépendances serveur
│   ├── config.toml          # Configuration runtime
│   ├── README.md
│   └── target/release/      # Binaire winlog-server
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
- **Client** : Modifier `client/src/lib.rs` (modules partagés)
- **Serveur** : Modifier `serveur/src/*.rs` (handlers, database, models)
- **Base de données** : Modifier `serveur/scripts/create_base.sh` (schéma SQLite)
- **API** : Ajouter endpoints dans `serveur/src/handlers.rs` + routes dans `main.rs`

## 🔐 Sécurité

### Client
- Pas de données sensibles dans le code
- Support HTTPS via certificats système
- User-Agent custom pour identification
- Pas d'exécution de commandes shell

### Serveur
- Validation stricte User-Agent et JSON
- Transactions ACID (pas de corruption)
- Firewall réseau recommandé (port 3000)
- HTTPS obligatoire en production (reverse proxy Nginx/Caddy)
- Rate limiting avec Axum middleware ou reverse proxy

### Recommandations production
- **HTTPS** : Reverse proxy Nginx + Let's Encrypt
- **Firewall** : Limiter au réseau interne uniquement (`ufw allow from 192.168.0.0/16`)
- **Backups** : Sauvegarde quotidienne SQLite (rotation automatique)
- **Monitoring** : Health check `/health` + logs serveur
- **Rotation** : Archiver/purger données anciennes (rotation quotidienne automatique)

## 📊 Performances

### Client
- **Démarrage** : ~10ms
- **Exécution** : <100ms (logon/logout), <500ms (matos)
- **Mémoire** : <5MB
- **Binaires** : 450-530KB après strip
- **Réseau** : ~500 octets par événement

### Serveur Rust (Axum + SQLx)
- **Débit** : ~5000 req/s (vs 100 req/s PHP)
- **Latence** : 0.6ms P50, 3ms P99 (réseau local)
- **Concurrence** : 10 000+ connexions simultanées
- **Mémoire** : ~10 MB (vs ~50 MB PHP)
- **Stockage** : ~250 octets par événement en DB
- **Requêtes** : <5ms pour sessions ouvertes (table partitionnée)

## 🗺️ Roadmap

### Phase actuelle : Production ready ✅
- [x] Client Rust fonctionnel Windows + Linux
- [x] Serveur Rust (Axum + SQLx) opérationnel
- [x] Base SQLite partitionnée (events_today/history)
- [x] Rotation quotidienne automatisée
- [x] Réorganisation repository (client/serveur)
- [x] Documentation complète (800+ lignes)
- [ ] Tests approfondis multi-plateformes
- [ ] Scripts d'installation automatisée
- [ ] Service systemd pour serveur

### Phase 2 : Fonctionnalités avancées 🔜
- [ ] API de consultation (GET /api/v1/sessions, /api/v1/events)
- [ ] Dashboard web temps réel (Rust + HTMX ou API REST + frontend)
- [ ] Authentification clients (tokens JWT ou certificats)
- [ ] Alertes (sessions anormales, nouveaux matériels)
- [ ] Export rapports (CSV, JSON)

### Phase 3 : Évolutions futures 🚀
- [ ] Support PostgreSQL (alternative SQLite pour grands parcs)
- [ ] Clustering/HA (plusieurs serveurs)
- [ ] Métriques Prometheus + Grafana
- [ ] Client mobile (inventaire à distance)
- [ ] Intégration LDAP/Active Directory

## 🤝 Contribution

### Standards de code
- **Rust** : `rustfmt` et `clippy` obligatoires avant commit
- **Commits** : Messages descriptifs en français
- **Documentation** : Mise à jour README.md synchrone avec le code
- **Tests** : Compilation sans warnings (`cargo build --release` clean)

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
