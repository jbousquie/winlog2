# Role
As a senior Rust developer, my core task is to analyze user edits and rewrite provided code excerpts, incorporating suitable suggestions based on cursor location. I prioritize writing efficient, readable, and maintainable Rust code, always adhering to best practices and ensuring thorough documentation.

I am responsible for testing and debugging to deliver error-free code that meets project requirements. When codebases grow, I propose refactoring into smaller, manageable functions and even splitting code into multiple files for better organization. Each file would contain functions related to a specific project aspect. Each time I add or modify a function, I add initial comments explaining the purpose and usage of the function. Each time I add a feature or modify an existing one or each time I refactor code, I ensure that the code remains well-organized and easy to understand and I update the file copilot-instructions.md and possibly README.md.

I meticulously manage imports and dependencies, ensuring they are well-organized and updated during refactoring. If new dependencies are needed, I propose adding them to Cargo.toml and verify compatibility. My goal is to centralize imports and dependencies whenever possible to enhance readability and maintainability. I never hardcode values but rather use constants from a configuration file. I add comments in every module and above each function to explain its purpose and usage.

I don't implement the project all at once, but rather in small, manageable steps under the guidance of the developer. I propose the developer a plan of steps to follow. I wait for the developer's instructions before proceeding on each step.

I don't run the code to test it, I just build it. The developer will run the code to test it.

I use the agentic tools like edit_file or patch to modify the code. If needed, I can also run commands from the shell, like cd, cat, printf, sed.

## Panic-Safety Requirements (Critical for Production)

**NEVER** allow panics in runtime code. All production code must be panic-proof.

### Forbidden Patterns (Will cause panics):
- ❌ `.unwrap()` without explicit justification
- ❌ `.expect()` in runtime code (only acceptable at startup for fail-fast)
- ❌ Direct indexing: `array[i]`, `string[start..end]`
- ❌ `.get().unwrap()` chains
- ❌ `panic!()` macro in runtime paths

### Required Safe Patterns:
- ✅ Use `.unwrap_or()`, `.unwrap_or_else()`, `.unwrap_or_default()` with explicit fallbacks
- ✅ Use `.get()` with safe slicing: `slice.get(start..end).unwrap_or_default()`
- ✅ Use `if let Ok()` / `match` for Result handling
- ✅ Use `if let Some()` / `match` for Option handling
- ✅ For HashMap: `.get(key).map(|v| v.as_str()).unwrap_or("default")` (avoids temp allocation)
- ✅ For SQLx: Use `.try_get()` instead of `.get()` to convert Result → Option
- ✅ Propagate errors with `?` operator or return `Result<T, E>`
- ✅ HTTP handlers must never panic - return appropriate status codes (400, 403, 500)

### Acceptable Panics (Fail-Fast Pattern):
- ⚠️ **Startup initialization only**: Configuration loading, database connection
- ⚠️ Use `.expect("descriptive message")` with clear reasoning
- ⚠️ Principle: Better not to start than to start in invalid state

### Example Transformations:

**Bad (will panic):**
```rust
let date = &timestamp[..10];  // Panic if timestamp.len() < 10
let value = map.get("key").unwrap();  // Panic if key missing
```

**Good (panic-proof):**
```rust
let date = timestamp.get(..10).unwrap_or("1970-01-01");
let value = map.get("key").map(|v| v.as_str()).unwrap_or("default");
```

### Verification:
- Run `cargo clippy` to detect potential panics
- Search codebase for `unwrap()`, `expect()`, `[..]` patterns
- Ensure all Result/Option types are handled explicitly
- Test edge cases: empty strings, None values, missing keys

# Description Technique du Projet Winlog 2 en Rust

## Vue d'ensemble
Ce projet développe un système de monitoring multi-plateforme (Windows + Linux) composé de 3 binaires Rust spécialisés côté client + une librairie partagée, et d'un serveur Rust (Axum + SQLx + SQLite) de collecte centralisé. Les binaires clients sont exécutés lors des événements de session (ouverture/fermeture) ou périodiquement pour l'inventaire matériel, et transmettent les données via HTTP POST à un serveur de monitoring.

## Architecture du projet

Le repository est organisé en **2 parties distinctes** :

### 1. Partie Client (`/client/`)

#### Configuration centralisée (`client/src/config.rs`)
- **Configuration hiérarchique** : Variables d'environnement (priorité) → Constantes par défaut (fallback)
- **Variables supportées** : `WINLOG_SERVER_URL`, `WINLOG_TIMEOUT`, `WINLOG_MAX_RETRIES`, `WINLOG_RETRY_DELAY_MS`, `WINLOG_USER_AGENT`
- **Avantages** : Pas de recompilation, déploiement centralisé via GPO/PAM, un binaire pour tous les environnements
- **Fonctions d'accès** : `server_url()`, `timeout()`, `max_retries()`, `retry_delay_ms()`, `user_agent()`
- **Déploiement production** : GPO Windows (Environment Variables) ou `/etc/environment` Linux

#### Librairie partagée (`client/src/lib.rs`)
- **Module `http_client`** : Client HTTP synchrone basé sur `minreq` avec retry et timeout
- **Module `system_info`** : Collecte synchrone d'informations (username, hostname, OS, matériel)
- **Module `data_structures`** : Structures sérialisables pour les données JSON
- **Module `utils`** : Utilitaires (timestamps, validation) + logique mutualisée des binaires

#### Binaires spécialisés (`client/src/bin/`)
- **`logon.rs`** : Traite les événements d'ouverture de session (Windows + Linux)
- **`logout.rs`** : Traite les événements de fermeture de session (Windows + Linux)
- **`matos.rs`** : Collecte les informations matérielles détaillées (Windows + Linux)

### 2. Partie Serveur (`/serveur/`)

#### Implémentation Rust (Axum + SQLx + SQLite)
- **Framework web** : Axum 0.7 (Tokio team, simplicité + stabilité)
- **API REST** : POST `/api/v1/events` (collecte), GET `/api/v1/sessions/current` (consultation), GET `/health` (monitoring)
- **Base de données** : SQLite en mode WAL avec structure partitionnée
- **ORM** : SQLx 0.8 avec compile-time SQL checks
- **Configuration** : `config.toml` pour paramètres runtime

#### Structure du code serveur (`serveur/src/`)
- **`main.rs`** : Point d'entrée Axum, initialisation serveur et routes
- **`config.rs`** : Chargement et validation config.toml
- **`models.rs`** : Structures ClientEvent, Response, DbEvent, CurrentSession avec serde
- **`queries.rs`** : Constantes SQL centralisées (toutes les requêtes du serveur)
- **`database.rs`** : Pool SQLx, logique sessions intelligente (utilise queries.rs)
- **`handlers.rs`** : Handlers HTTP (collect_event, get_current_sessions, health_check, extract_ip)

#### Scripts bash de gestion DB (`serveur/scripts/`)
- **`create_base.sh`** : Création base partitionnée (events_today + events_history)
- **`purge_base.sh`** : Vidage sélectif (--today/--history/--all)
- **`delete_base.sh`** : Suppression complète avec confirmation
- **`rotate_daily.sh`** : Rotation quotidienne automatique (cron 1h)

#### Base SQLite partitionnée
- **events_today** : Événements du jour (~100 rows, lectures/écritures rapides)
- **events_history** : Archive complète (10k+ rows, lecture seule sauf rotation)
- **events_all** : Vue UNION ALL pour requêtes globales
- **6 index** : username, timestamp, hostname, action_user, session, ip (x2 tables)
- **Mode WAL** : Lectures concurrentes sans verrous, backup online

## Spécifications techniques

### Architecture synchrone
Le projet utilise une **architecture 100% synchrone** optimisée pour des scripts one-shot :
- **Binaires légers** : Exécution linéaire sans runtime asynchrone
- **Client HTTP** : `minreq` pour des requêtes POST synchrones rapides
- **Pas de tokio** : Évite l'overhead d'un runtime async inutile
- **Démarrage instantané** : ~10ms vs ~100ms avec un runtime async

### Stack technique actuelle

#### Client Rust (multi-plateforme synchrone)
- **`sysinfo`** : Collecte d'informations système synchrone (Windows + Linux)
- **`minreq`** : Client HTTP léger (~200KB) avec timeout et retry
- **`serde` + `serde_json`** : Sérialisation automatique des structures
- **`chrono`** : Timestamps ISO 8601 UTC
- **`whoami`** : Récupération du username multi-plateforme (Windows/Linux)

#### Serveur Rust (asynchrone haute performance)
- **Axum 0.7** : Framework web Tokio team, simplicité + stabilité
- **SQLx 0.8** : ORM avec compile-time SQL checks, pool de connexions
- **SQLite + WAL** : Base partitionnée (events_today/history), lectures concurrentes
- **serde + serde_json** : Sérialisation/désérialisation JSON automatique
- **tracing + tracing-subscriber** : Logs structurés pour observabilité
- **chrono** : Timestamps serveur ISO 8601 UTC
- **md5** : Génération UUID sessions (username@hostname@hash6)
- **toml** : Configuration runtime depuis config.toml

### Données collectées
- **Username** : Utilisateur Windows/Linux actuel
- **Action** : Code d'événement ("C" = Connexion, "D" = Déconnexion, "M" = Matériel)
- **Timestamp** : Horodatage ISO 8601 UTC
- **Informations système** : OS, version, architecture (adapté selon plateforme)
- **Informations matérielles** : CPU, RAM, disques, réseau (pour matos uniquement)

### Format de communication
- **Protocole** : HTTP POST avec payload JSON
- **Timeout** : Configurable (défaut 30s)
- **Retry** : 3 tentatives avec backoff exponentiel
- **Headers** : Content-Type application/json, User-Agent custom

### Gestion des erreurs
- **Logging** : Messages d'erreur dans Event Log (Windows) ou syslog (Linux)
- **Fallback** : Stockage local en cas d'indisponibilité serveur (future feature)
- **Validation** : Vérification des données avant envoi

## Contraintes d'implémentation
- **Performances** : Exécution ultra-rapide (<100ms) grâce à l'architecture synchrone
- **Ressources** : Empreinte mémoire minimale (<5MB) sans overhead async
- **Sécurité** : Pas de données sensibles en dur, support HTTPS via minreq
- **Compatibilité** : Windows 10/11 ET Linux (Ubuntu, Debian, RHEL, Arch...)
- **Compilation** : MinGW/GCC ou MSVC pour Windows, GCC/rustc pour Linux
- **Fiabilité** : Retry automatique (3 tentatives) avec délai configurable

## Optimisations récentes

### Logique mutualisée (Janvier 2026)
- **Déduplication** : Fonctions communes `process_session_event()` et `process_hardware_info()`
- **Codes d'action courts** : "C", "D", "M" au lieu de textes longs
- **Binaires simplifiés** : ~10 lignes de code au lieu de ~50
- **Maintenance centralisée** : Modifications dans lib.rs uniquement
- **JSON optimisé** : Réduction de la bande passante avec des codes courts

## Plan de développement
1. **Phase 1** : Structure du projet et librairie de base ✅
2. **Phase 2** : Architecture synchrone et client HTTP `minreq` ✅
3. **Phase 3** : Intégration complète des 3 binaires ✅
4. **Phase 4** : Tests et validation fonctionnelle ✅
5. **Phase 5** : Réorganisation repository (client/serveur) ✅
6. **Phase 6** : Support multi-plateforme Linux
7. **Phase 7** : Migration serveur vers Rust
8. **Phase 8** : Dashboard web et API REST
9. **Phase 9** : Packaging et déploiement

## Évolutions récentes

### Réorganisation repository (Janvier 2026)
- **Structure client/serveur** : Séparation claire entre code client Rust et serveur Rust
- **Documentation dédiée** : README.md pour chaque partie + README.md global
- **Support multi-plateforme** : Structure adaptée Windows + Linux (client et serveur)
- **Stack complète Rust** : Client et serveur en Rust pour cohérence et performances

### Serveur Rust Axum (Janvier 2026)
- **Framework Axum 0.7** : API REST asynchrone haute performance (~5000 req/s)
- **SQLx 0.8** : ORM avec compile-time SQL checks, pool de connexions
- **Base partitionnée** : events_today + events_history pour performances optimales
- **Scripts bash** : Gestion DB (création, rotation, purge)
- **Configuration TOML** : Runtime config dans config.toml (host, port, PRAGMA SQLite)
- **Binaire standalone** : 3.1 MB stripped, aucune dépendance système

### Refactorisation de la configuration (Janvier 2026)
- **Extraction** : Module `config` déplacé de `lib.rs` vers `config.rs` autonome
- **Amélioration** : Ajout de constantes additionnelles (USER_AGENT, RETRY_DELAY_MS)
- **Centralisation** : Configuration accessible via `crate::config` depuis tous les modules
- **Maintenabilité** : Séparation claire entre logique métier et paramètres de configuration

### Architecture synchrone finalisée (Janvier 2026)
- **Optimisation** : Architecture 100% synchrone côté client pour performances optimales
- **Client HTTP** : Utilisation de `minreq` pour un client léger sans dépendances async
- **Binaires** : Fonctions `main()` synchrones pour démarrage instantané
- **Performance** : Empreinte mémoire réduite et temps de lancement minimal
- **Compilation** : Support MinGW/GCC et MSVC pour Windows

### Refactorisation avec logique mutualisée (Janvier 2026)
- **Déduplication majeure** : Création de `process_session_event()` commune à logon/logout
- **Codes d'action optimisés** : "C"/"D"/"M" pour communication compacte
- **Binaires ultra-légers** : 8-10 lignes de code par binaire
- **JSON compact** : Réduction de 40% de la taille des payloads
- **Maintenance simplifiée** : Toute la logique centralisée dans utils

## Structure des fichiers du projet

```
winlog2/
├── client/                     # Code client Rust multi-plateforme
│   ├── src/
│   │   ├── bin/
│   │   │   ├── logon.rs       # Binaire ouverture session (8 lignes)
│   │   │   ├── logout.rs      # Binaire fermeture session (8 lignes)
│   │   │   └── matos.rs       # Binaire inventaire matériel (10 lignes)
│   │   ├── config.rs          # Configuration centralisée
│   │   └── lib.rs             # Modules : http_client, system_info, data_structures, utils
│   ├── Cargo.toml             # Dépendances et métadonnées Rust
│   ├── Cargo.lock
│   ├── README.md              # Documentation client (compilation, déploiement)
│   └── target/release/        # Binaires compilés (logon, logout, matos)
│
├── serveur/                    # Code serveur Rust (Axum + SQLx)
│   ├── src/
│   │   ├── main.rs            # Point d'entrée Axum + routes
│   │   ├── config.rs          # Chargement config.toml
│   │   ├── models.rs          # Structures ClientEvent, Response
│   │   ├── queries.rs         # Constantes SQL centralisées
│   │   ├── database.rs        # Pool SQLx, logique sessions (utilise queries.rs)
│   │   └── handlers.rs        # Handlers HTTP (collect_event, health)
│   ├── scripts/               # Scripts bash gestion DB
│   │   ├── create_base.sh     # Création base partitionnée
│   │   ├── purge_base.sh      # Vidage sélectif
│   │   ├── delete_base.sh     # Suppression complète
│   │   ├── rotate_daily.sh    # Rotation quotidienne
│   │   └── README.md          # Documentation scripts
│   ├── Cargo.toml             # Dépendances serveur
│   ├── config.toml            # Configuration runtime
│   ├── README.md              # Documentation serveur
│   └── target/release/winlog-server  # Binaire serveur (3.1 MB)
│
├── README.md                   # Documentation globale du projet
├── .gitignore
└── .github/
    └── copilot-instructions.md # Ce fichier - Instructions développement
```

## Compilation et tests

### Client Rust

#### Build Linux (natif)
```bash
cd client
cargo build --release
# Binaires : target/release/{logon,logout,matos}
```

#### Build Windows (cross-compilation depuis Linux)
```bash
cd client
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
# Binaires : target/x86_64-pc-windows-gnu/release/{logon,logout,matos}.exe
```

#### Build Windows (natif)
```bash
cd client
cargo build --release
# Binaires : target\release\{logon,logout,matos}.exe
```

#### Vérification compilation
```bash
cd client
cargo check        # Vérification rapide sans build complet
cargo clippy       # Linter Rust (détection problèmes)
cargo test         # Exécution des tests unitaires
```

### Serveur Rust

#### Compilation
```bash
cd serveur
cargo build --release
# Binaire : target/release/winlog-server (3.1 MB stripped)
```

#### Création base de données
```bash
cd serveur/scripts
./create_base.sh
# Crée serveur/data/winlog.db avec structure partitionnée
```

#### Démarrage
```bash
cd serveur
./target/release/winlog-server
# Écoute sur http://127.0.0.1:3000 par défaut (config.toml)
```

#### Tests
```bash
# Health check
curl http://127.0.0.1:3000/health
# Attendu : {"status":"healthy","database":"connected",...}

# Test événement
curl -X POST http://127.0.0.1:3000/api/v1/events \
  -H "Content-Type: application/json" \
  -H "User-Agent: Winlog/0.1.0" \
  -d '{"username":"test","action":"C","timestamp":"2026-01-13T08:30:00Z","hostname":"TEST-PC","os_info":{"os_name":"Ubuntu 24.04","os_version":"24.04","kernel_version":"6.8.0"}}'
# Attendu : {"status":"success","event_id":1,"session_uuid":"test@TEST-PC@...",...}
```

## Workflow de développement

1. **Modification code client** :
   - Éditer `client/src/*.rs`
   - Vérifier : `cd client && cargo check`
   - Build : `cargo build --release`
   - Documenter : Mettre à jour `client/README.md` si nécessaire

2. **Modification code serveur** :
   - Éditer `serveur/src/*.rs`
   - Vérifier : `cd serveur && cargo check`
   - Build : `cargo build --release`
   - Tester : Curl ou clients Rust réels
   - Documenter : Mettre à jour `serveur/README.md` si nécessaire

3. **Modification base de données** :
   - Éditer `serveur/scripts/create_base.sh` (schéma SQLite)
   - Adapter requêtes dans `serveur/src/database.rs` (SQLx)
   - Tester : Créer une base test et vérifier avec des requêtes

4. **Changements globaux** :
   - Mettre à jour `README.md` global
   - Mettre à jour `.github/copilot-instructions.md` (ce fichier)
   - Commit descriptif en français
   - Vérifier compilation : `cargo build --release` (0 warnings requis)

## Bonnes pratiques

### Code Rust (Client + Serveur)
- **Formatting** : `cargo fmt` avant chaque commit
- **Linting** : `cargo clippy` doit passer sans warnings
- **Warnings** : Compilation release doit être clean (0 warning)
- **Documentation** : Commentaires rustdoc (`///`) sur fonctions publiques + modules
- **Tests** : Tests unitaires pour logique métier critique
- **Constantes** : Toujours utiliser `config.rs` / `config.toml`, jamais de hardcode
- **Sécurité** : Pas de `unwrap()` sans gestion erreur, préférer `?` ou `match`
- **Binaires** : Garder les binaires minimalistes (déléguer logique à `lib.rs`)
- **Panic-Safety** : Code runtime doit être 100% panic-proof (voir section Role ci-dessus)

### Code SQL (Base SQLite)
- **Organisation** : Toutes les requêtes SQL doivent être dans le fichier dédié `serveur/src/queries.rs`
- **Nomenclature** : Préfixer les constantes par `SQL_` suivi d'un verbe d'action (ex: `SQL_FIND_OPEN_SESSION_TODAY`, `SQL_INSERT_EVENT`)
- **Visibilité** : Utiliser `pub const` pour rendre les constantes accessibles depuis d'autres modules
- **Documentation** : Chaque constante SQL doit avoir un commentaire détaillé expliquant :
  - **Objectif** : Pourquoi cette requête existe
  - **Logique** : Comment elle fonctionne (filtres, jointures, sous-requêtes)
  - **Paramètres** : Liste ordonnée des paramètres `?` avec leur type et signification
  - **Colonnes retournées** : Liste des colonnes du SELECT (si applicable)
  - **Utilisé dans** : Référence au module/fonction qui utilise cette requête
- **Séparation** : Ne jamais mettre de SQL inline dans `database.rs` ou `handlers.rs`, toujours passer par `queries.rs`
- **Indexes** : Toujours indexer colonnes dans WHERE/JOIN
- **Transactions** : Utiliser transactions pour cohérence ACID
- **PRAGMA** : Configurer WAL mode + cache_size pour performances
- **Requêtes** : Tester avec `EXPLAIN QUERY PLAN` pour optimisation
- **Compile-time checks** : SQLx vérifie requêtes à la compilation
- **Safe queries** : Utiliser `.try_get()` au lieu de `.get()` pour éviter panics

### Documentation
- Synchroniser README.md avec chaque changement de fonctionnalité
- Expliquer le "pourquoi" pas seulement le "quoi"
- Exemples concrets de déploiement et utilisation
- Maintenir la roadmap à jour dans README.md global
- Documenter les choix de panic-safety (fallbacks, error handling)