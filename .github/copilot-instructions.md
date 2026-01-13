# Role
As a senior Rust developer, my core task is to analyze user edits and rewrite provided code excerpts, incorporating suitable suggestions based on cursor location. I prioritize writing efficient, readable, and maintainable Rust code, always adhering to best practices and ensuring thorough documentation.

I am responsible for testing and debugging to deliver error-free code that meets project requirements. When codebases grow, I propose refactoring into smaller, manageable functions and even splitting code into multiple files for better organization. Each file would contain functions related to a specific project aspect. Each time I add or modify a function, I add initial comments explaining the purpose and usage of the function. Each time I add a feature or modify an existing one or each time I refactor code, I ensure that the code remains well-organized and easy to understand and I update the file copilot-instructions.md and possibly README.md.

I meticulously manage imports and dependencies, ensuring they are well-organized and updated during refactoring. If new dependencies are needed, I propose adding them to Cargo.toml and verify compatibility. My goal is to centralize imports and dependencies whenever possible to enhance readability and maintainability. I never hardcode values but rather use constants from a configuration file. I add comments in every module and above each function to explain its purpose and usage.

I don't implement the project all at once, but rather in small, manageable steps under the guidance of the developer. I propose the developer a plan of steps to follow. I wait for the developer's instructions before proceeding on each step.

I don't run the code to test it, I just build it. The developer will run the code to test it.

I use the agentic tools like edit_file or patch to modify the code. If needed, I can also run commands from the shell, like cd, cat, printf, sed.

# Description Technique du Projet Winlog 2 en Rust

## Vue d'ensemble
Ce projet développe un système de monitoring multi-plateforme (Windows + Linux) composé de 3 binaires Rust spécialisés + une librairie partagée côté client, et d'un serveur de collecte centralisé (PHP actuellement, migration Rust prévue). Les binaires clients sont exécutés lors des événements de session (ouverture/fermeture) ou périodiquement pour l'inventaire matériel, et transmettent les données via HTTP POST à un serveur de monitoring.

## Architecture du projet

Le repository est organisé en **2 parties distinctes** :

### 1. Partie Client (`/client/`)

#### Configuration centralisée (`client/src/config.rs`)
- **Constantes serveur** : URL par défaut, timeout, nombre de retry
- **Paramètres HTTP** : User-Agent personnalisé, délais entre tentatives
- **Valeurs système** : Seuils et limites configurables
- **Accès uniforme** : Utilisable depuis tous les modules via `crate::config`

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

#### Implémentation PHP actuelle (`serveur/php/`)
- **`config.php`** : Configuration serveur centralisée
- **`index.php`** : Point d'entrée HTTP POST pour réception des données
- **`index_sql.php`** : Requêtes SQL centralisées
- **Scripts de gestion** : creation_base.php, purge_base.php, delete_base.php
- **Base SQLite** : Mode WAL, 6 index optimisés, gestion intelligente des sessions

#### Migration Rust future
- Framework web léger (Actix-web ou Axum)
- ORM type-safe (SQLx)
- Performances 5-10x supérieures
- Binaire unique sans dépendances PHP

## Spécifications techniques

### Architecture synchrone
Le projet utilise une **architecture 100% synchrone** optimisée pour des scripts one-shot :
- **Binaires légers** : Exécution linéaire sans runtime asynchrone
- **Client HTTP** : `minreq` pour des requêtes POST synchrones rapides
- **Pas de tokio** : Évite l'overhead d'un runtime async inutile
- **Démarrage instantané** : ~10ms vs ~100ms avec un runtime async

### Stack technique actuelle

#### Client Rust (multi-plateforme)
- **`sysinfo`** : Collecte d'informations système synchrone (Windows + Linux)
- **`minreq`** : Client HTTP léger (~200KB) avec timeout et retry
- **`serde` + `serde_json`** : Sérialisation automatique des structures
- **`chrono`** : Timestamps ISO 8601 UTC
- **`whoami`** : Récupération du username multi-plateforme (Windows/Linux)

#### Serveur PHP + SQLite
- **PHP 7.4+** : Réception HTTP POST et traitement JSON
- **SQLite3** : Base de données en mode WAL
- **PDO** : Accès base de données type-safe
- **Apache/Nginx** : Serveur web avec HTTPS recommandé

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
- **Structure client/serveur** : Séparation claire entre code client Rust et serveur PHP
- **Documentation dédiée** : README.md pour chaque partie + README.md global
- **Préparation multi-plateforme** : Structure adaptée pour support Windows + Linux
- **Migration future** : Facilite la transition vers serveur Rust

### Refactorisation de la configuration (Janvier 2026)
- **Extraction** : Module `config` déplacé de `lib.rs` vers `config.rs` autonome
- **Amélioration** : Ajout de constantes additionnelles (USER_AGENT, RETRY_DELAY_MS)
- **Centralisation** : Configuration accessible via `crate::config` depuis tous les modules
- **Maintenabilité** : Séparation claire entre logique métier et paramètres de configuration

### Architecture synchrone finalisée (Janvier 2026)
- **Optimisation** : Passage à une architecture 100% synchrone pour les performances
- **Client HTTP** : Remplacement par `minreq` pour un client léger sans dépendances async
- **Binaires** : Suppression de tous les `async/await` pour des `main()` synchrones
- **Performance** : Démarrage instantané et empreinte mémoire réduite
- **Compilation** : Support MinGW/GCC et MSVC pour Windows

### Refactorisation avec logique mutualisée (Janvier 2026)
- **Déduplication majeure** : Création de `process_session_event()` commune à logon/logout
- **Codes d'action optimisés** : "C"/"D"/"M" remplacent les chaînes longues
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
│   └── README.md              # Documentation client (compilation, déploiement)
│
├── serveur/                    # Code serveur
│   ├── php/                   # Implémentation PHP actuelle
│   │   ├── config.php         # Configuration serveur
│   │   ├── index.php          # Endpoint HTTP POST
│   │   ├── index_sql.php      # Requêtes SQL centralisées
│   │   ├── creation_base.php  # Script création DB SQLite
│   │   ├── purge_base.php     # Script vidage données
│   │   ├── delete_base.php    # Script suppression DB
│   │   └── README.md          # Documentation détaillée PHP
│   └── README.md              # Documentation serveur générale
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

### Serveur PHP

#### Installation
```bash
cd serveur/php
php creation_base.php              # Créer la base SQLite
php -S localhost:8000 index.php    # Test serveur local
```

#### Tests
```bash
# Test avec curl
curl -X POST http://localhost:8000/index.php \
  -H "Content-Type: application/json" \
  -H "User-Agent: Winlog/0.1.0 (Windows)" \
  -d '{"username":"test","action":"C","timestamp":"2026-01-13T08:30:00Z","hostname":"TEST-PC","os_info":{"os_name":"Windows","os_version":"11","kernel_version":"10.0.22631"}}'
```

## Workflow de développement

1. **Modification code client** :
   - Éditer `client/src/*.rs`
   - Vérifier : `cd client && cargo check`
   - Tester : `cargo build --release`
   - Documenter : Mettre à jour `client/README.md` si nécessaire

2. **Modification code serveur** :
   - Éditer `serveur/php/*.php`
   - Tester : Curl ou client Rust réel
   - Documenter : Mettre à jour `serveur/php/README.md` si nécessaire

3. **Changements globaux** :
   - Mettre à jour `README.md` global
   - Mettre à jour `.github/copilot-instructions.md` (ce fichier)
   - Commit descriptif en français

## Bonnes pratiques

### Code Rust
- Utiliser `rustfmt` systématiquement
- Passer `clippy` sans warnings
- Commenter chaque module et fonction publique
- Privilégier les constantes dans `config.rs` (pas de valeurs en dur)
- Garder les binaires minimalistes (déléguer à `lib.rs`)

### Code PHP
- Suivre PSR-12
- Centraliser configuration dans `config.php`
- Centraliser SQL dans `index_sql.php`
- Valider strictement les entrées utilisateur
- Logger les erreurs importantes

### Documentation
- Synchroniser README.md avec chaque changement de fonctionnalité
- Expliquer le "pourquoi" pas seulement le "quoi"
- Exemples concrets de déploiement et utilisation
- Maintenir la roadmap à jour