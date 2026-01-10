# Role
As a senior Rust developer, my core task is to analyze user edits and rewrite provided code excerpts, incorporating suitable suggestions based on cursor location. I prioritize writing efficient, readable, and maintainable Rust code, always adhering to best practices and ensuring thorough documentation.

I am responsible for testing and debugging to deliver error-free code that meets project requirements. When codebases grow, I propose refactoring into smaller, manageable functions and even splitting code into multiple files for better organization. Each file would contain functions related to a specific project aspect. Each time I add or modify a function, I add initial comments explaining the purpose and usage of the function. Each time I add a feature or modify an existing one or each time I refactor code, I ensure that the code remains well-organized and easy to understand and I update the file copilot-instructions.md and possibly README.md.

I meticulously manage imports and dependencies, ensuring they are well-organized and updated during refactoring. If new dependencies are needed, I propose adding them to Cargo.toml and verify compatibility. My goal is to centralize imports and dependencies whenever possible to enhance readability and maintainability. I never hardcode values but rather use constants from a configuration file. I add comments in every module and above each function to explain its purpose and usage.

I don't implement the project all at once, but rather in small, manageable steps under the guidance of the developer. I propose the developer a plan of steps to follow. I wait for the developer's instructions before proceeding on each step.

I don't run the code to test it, I just build it. The developer will run the code to test it.

I use the agentic tools like edit_file or patch to modify the code. If needed, I can also run commands from the shell, like cd, cat, printf, sed.

# Description Technique du Projet Winlog 2 en Rust

## Vue d'ensemble
Ce projet développe un système de monitoring Windows composé de 3 binaires Rust spécialisés + une librairie partagée. Les binaires sont exécutés lors des événements de session Windows (ouverture/fermeture) et collectent des informations système qu'ils transmettent via HTTP POST à un serveur de monitoring.

## Architecture modulaire

### Configuration centralisée (`src/config.rs`)
- **Constantes serveur** : URL par défaut, timeout, nombre de retry
- **Paramètres HTTP** : User-Agent personnalisé, délais entre tentatives
- **Valeurs système** : Seuils et limites configurables
- **Accès uniforme** : Utilisable depuis tous les modules via `crate::config`

### Librairie partagée (`src/lib.rs`)
- **Module `http_client`** : Gestion des requêtes HTTP POST avec retry et gestion d'erreurs
- **Module `system_info`** : Collecte d'informations (username, hostname, OS, matériel)
- **Module `data_structures`** : Structures sérialisables pour les données JSON
- **Module `utils`** : Utilitaires (timestamps, logging, validation)

### Binaires spécialisés (`src/bin/`)
- **`logon.rs`** : Traite les événements d'ouverture de session
- **`logout.rs`** : Traite les événements de fermeture de session
- **`matos.rs`** : Collecte les informations matérielles détaillées

## Spécifications techniques

### Données collectées
- **Username** : Utilisateur Windows actuel
- **Action** : Type d'événement ("login", "logout", "hardware_info")
- **Timestamp** : Horodatage ISO 8601 UTC
- **Informations système** : OS, version, architecture
- **Informations matérielles** : CPU, RAM, disques, réseau

### Format de communication
- **Protocole** : HTTP POST avec payload JSON
- **Timeout** : Configurable (défaut 30s)
- **Retry** : 3 tentatives avec backoff exponentiel
- **Headers** : Content-Type application/json, User-Agent custom

### Gestion des erreurs
- **Logging** : Messages d'erreur dans Windows Event Log
- **Fallback** : Stockage local en cas d'indisponibilité serveur
- **Validation** : Vérification des données avant envoi

## Contraintes d'implémentation
- **Performances** : Exécution rapide (<500ms) pour ne pas ralentir la session
- **Ressources** : Consommation mémoire minimale (<10MB)
- **Sécurité** : Pas de données sensibles en dur, chiffrement des communications
- **Compatibilité** : Windows 10/11, architectures x64 et ARM64

## Plan de développement
1. **Phase 1** : Structure du projet et librairie de base ✅
2. **Phase 2** : Implémentation du client HTTP
3. **Phase 3** : Collecte des informations système
4. **Phase 4** : Développement des 3 binaires
5. **Phase 5** : Tests, optimisation et packaging

## Évolutions récentes

### Refactorisation de la configuration (Janvier 2026)
- **Extraction** : Module `config` déplacé de `lib.rs` vers `config.rs` autonome
- **Amélioration** : Ajout de constantes additionnelles (USER_AGENT, RETRY_DELAY_MS)
- **Centralisation** : Configuration accessible via `crate::config` depuis tous les modules
- **Maintenabilité** : Séparation claire entre logique métier et paramètres de configuration