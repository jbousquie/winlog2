# Winlog Client - Monitoring Multi-plateforme

Client de monitoring léger pour Windows et Linux, développé en Rust. Collecte et transmet les événements de session (connexion/déconnexion) et les informations matérielles vers un serveur centralisé.

## 🎯 Objectif

Monitorer en temps réel l'activité des utilisateurs sur un parc de machines hétérogène (Windows/Linux) via des binaires ultra-légers déployés sur chaque poste.

## 🏗️ Architecture

### 3 Binaires spécialisés

#### `logon` (Ouverture de session)
- **Plateforme** : Windows + Linux
- **Déclencheur** : Script d'ouverture de session (GPO Windows / PAM Linux)
- **Action** : Code "C" (Connexion)
- **Données** : Username, timestamp, hostname, OS, architecture

#### `logout` (Fermeture de session)
- **Plateforme** : Windows + Linux
- **Déclencheur** : Script de fermeture de session
- **Action** : Code "D" (Déconnexion)
- **Données** : Username, timestamp, durée de session

#### `matos` (Inventaire matériel)
- **Plateforme** : Windows + Linux
- **Déclencheur** : Tâche planifiée ou exécution manuelle
- **Action** : Code "M" (Matériel)
- **Données** : CPU, RAM, disques, réseau, périphériques

### Librairie partagée (`src/lib.rs`)

**Modules** :
- `config` : Configuration centralisée (URL serveur, timeouts, retry)
- `http_client` : Client HTTP synchrone avec retry automatique (minreq)
- `system_info` : Collecte d'informations système multi-plateforme (sysinfo)
- `data_structures` : Structures sérialisables JSON
- `utils` : Logique commune et fonctions mutualisées

## 🔧 Compilation

### Prérequis
- **Rust** : 1.70+ (recommandé : dernière stable)
- **Cibles** : 
  - Windows : `x86_64-pc-windows-gnu` ou `x86_64-pc-windows-msvc`
  - Linux : `x86_64-unknown-linux-gnu`

### Build multi-plateforme

```bash
# Sur Linux (pour Linux)
cargo build --release

# Sur Linux (pour Windows via MinGW)
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu

# Sur Windows (natif MSVC)
cargo build --release

# Sur Windows (MinGW/GCC)
cargo build --release --target x86_64-pc-windows-gnu
```

### Binaires générés
- **Linux** : `target/release/logon`, `target/release/logout`, `target/release/matos`
- **Windows** : `target/release/logon.exe`, `target/release/logout.exe`, `target/release/matos.exe`

## 📦 Dépendances

| Crate | Version | Rôle |
|-------|---------|------|
| `sysinfo` | 0.37.2 | Collecte système (CPU, RAM, OS) - Multi-plateforme |
| `minreq` | 2.14 | Client HTTP synchrone léger (~200KB) |
| `serde` + `serde_json` | 1.0 | Sérialisation JSON |
| `chrono` | 0.4 | Timestamps ISO 8601 UTC |
| `whoami` | 1.4 | Détection username (Windows/Linux) |

## 🚀 Architecture technique

### 100% Synchrone
- **Pas de runtime async** : Démarrage instantané (~10ms)
- **Exécution linéaire** : Collecte → Sérialisation → Envoi → Fin
- **Empreinte minimale** : <5MB RAM, binaires ~1MB
- **Optimisé one-shot** : Idéal pour scripts GPO/PAM

### Communication HTTP
- **Protocole** : HTTP POST avec payload JSON
- **Endpoint** : Configurable via `config::SERVER_URL`
- **Timeout** : 30s par défaut
- **Retry** : 3 tentatives avec backoff exponentiel (500ms, 1s, 2s)
- **Headers** : `Content-Type: application/json`, `User-Agent: Winlog/0.1.0`

### Format JSON

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
- `"C"` : Connexion (logon)
- `"D"` : Déconnexion (logout)
- `"M"` : Matériel (matos)

## 🖥️ Compatibilité multi-plateforme

### Windows (10/11)
- **Déploiement** : Stratégies de groupe (GPO)
- **Scripts** : Ouverture/Fermeture de session
- **Compilation** : MinGW (GCC) ou MSVC
- **Testée** : VM Windows 11 sur Ubuntu host

### Linux (Ubuntu, Debian, RHEL, Arch, etc.)
- **Déploiement** : Scripts PAM ou systemd user services
- **Compilation** : GCC/rustc natif
- **Testée** : Ubuntu 22.04+

### Différences plateforme
La crate `sysinfo` adapte automatiquement la collecte selon l'OS :
- **Windows** : API Win32, WMI
- **Linux** : `/proc`, `/sys`, udev

## 📝 Configuration

Modifier `src/config.rs` pour ajuster :
- `SERVER_URL` : Adresse du serveur Winlog
- `HTTP_TIMEOUT_SECS` : Timeout des requêtes (défaut : 30s)
- `MAX_RETRIES` : Nombre de tentatives (défaut : 3)
- `RETRY_DELAY_MS` : Délai entre tentatives (défaut : 500ms)
- `USER_AGENT` : User-Agent des requêtes

**Recompiler après modification** : `cargo build --release`

## 🚀 Déploiement

### Windows (GPO)
1. Copier `logon.exe` et `logout.exe` vers `\\DOMAIN\SYSVOL\scripts\`
2. Configurer GPO :
   - **Ouverture** : `User Configuration > Scripts > Logon > Add logon.exe`
   - **Fermeture** : `User Configuration > Scripts > Logoff > Add logout.exe`
3. Déployer `matos.exe` via tâche planifiée (quotidien)

### Linux (PAM)
1. Copier binaires vers `/usr/local/bin/`
2. Créer scripts wrappers :
   ```bash
   # /etc/profile.d/winlog-logon.sh
   /usr/local/bin/logon &
   
   # /etc/bash.bash_logout (ou ~/.bash_logout)
   /usr/local/bin/logout &
   ```
3. Tâche cron pour `matos` : `0 2 * * * /usr/local/bin/matos`

## 🔍 Tests et validation

```bash
# Vérifier la compilation
cargo check

# Build optimisé
cargo build --release

# Tester manuellement (remplacer URL serveur dans config.rs)
./target/release/logon
./target/release/logout
./target/release/matos
```

## 📊 Performances

- **Démarrage** : ~10ms (architecture synchrone)
- **Exécution** : <100ms (collecte + envoi HTTP)
- **Mémoire** : <5MB pendant exécution
- **Taille binaires** : ~800KB-1.2MB (après strip)
- **Réseau** : ~500 octets par événement (JSON compressé)

## 🛠️ Développement

### Structure des fichiers
```
client/
├── src/
│   ├── bin/
│   │   ├── logon.rs      # Binaire ouverture session
│   │   ├── logout.rs     # Binaire fermeture session
│   │   └── matos.rs      # Binaire inventaire matériel
│   ├── config.rs         # Configuration centralisée
│   └── lib.rs            # Librairie partagée (modules)
├── Cargo.toml            # Métadonnées et dépendances
└── README.md             # Cette documentation
```

### Logique mutualisée
Les binaires utilisent des fonctions communes de `src/lib.rs::utils` :
- `process_session_event(action_code)` : Logique logon/logout
- `process_hardware_info()` : Logique matos
- Validation, retry, gestion d'erreurs centralisée

### Ajout de fonctionnalités
1. Modifier `src/lib.rs` (modules concernés)
2. Tester avec `cargo check` et `cargo test`
3. Recompiler : `cargo build --release`
4. Mettre à jour cette documentation

## 🔐 Sécurité

- **Pas de données sensibles** : Aucun mot de passe, hash ou clé en clair
- **HTTPS supporté** : Via minreq (certificats système)
- **User-Agent custom** : Identification serveur-side
- **Validation JSON** : Côté serveur pour éviter injection
- **Pas de shell** : Aucune exécution de commandes externes

## 📖 Documentation complète

Voir la documentation globale du projet dans `/README.md` et la documentation serveur dans `/serveur/README.md`.
