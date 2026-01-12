# Winlog 2

Réimplémentation en Rust du client Winlog joué à l'ouverture ou à la fermeture d'une session Windows pour récupérer le username, l'action (ouverture/fermeture), l'horodatage et des informations sur le système et les envoyer par un POST HTTP vers un serveur de monitoring.

## Architecture du projet

Le projet est organisé autour de **3 binaires spécialisés** partageant une **librairie commune** :

### Structure des fichiers
```
src/
├── config.rs           # Configuration centralisée du projet
├── lib.rs              # Librairie partagée (logique commune)
├── bin/
│   ├── logon.rs        # Binaire pour ouverture de session
│   ├── logout.rs       # Binaire pour fermeture de session
│   └── matos.rs        # Binaire pour informations matérielles
```

### Les 3 binaires

#### `logon.exe`
- **Rôle** : Exécuté lors de l'ouverture de session Windows
- **Code d'action** : "C" (Connexion)
- **Données collectées** : Username, timestamp, informations système de base
- **Comportement** : Envoi immédiat des données au serveur

#### `logout.exe`
- **Rôle** : Exécuté lors de la fermeture de session Windows
- **Code d'action** : "D" (Déconnexion)  
- **Données collectées** : Username, timestamp, durée de session
- **Comportement** : Envoi immédiat des données au serveur

#### `matos.exe`
- **Rôle** : Collecte d'informations détaillées sur le matériel
- **Code d'action** : "M" (Matériel)
- **Données collectées** : Spécifications complètes du système (CPU, RAM, disques, etc.)
- **Comportement** : Peut être exécuté périodiquement ou sur demande

### Librairie partagée (`lib.rs`) et Configuration (`config.rs`)

**Configuration centralisée (`config.rs`)** :
- **Constantes serveur** : URL par défaut, timeout, retry
- **Paramètres HTTP** : User-Agent, délais de retry  
- **Configuration système** : Limites et seuils

**Modules de la librairie (`lib.rs`)** :

- **Client HTTP synchrone** : Utilise `minreq` pour des requêtes POST rapides avec retry automatique
- **Collecte système** : Récupération synchrone des informations (username, hostname, OS, matériel)
- **Structures JSON** : Données sérialisables avec codes d'action optimisés ("C", "D", "M")
- **Utilitaires communs** : Logique mutualisée pour les événements de session et collecte matérielle
- **Validation** : Contrôles de cohérence avant envoi

### Architecture synchrone optimisée

Les binaires utilisent une architecture **100% synchrone** optimisée pour des scripts d'exécution rapide :
- **Démarrage instantané** : Pas d'overhead de runtime asynchrone
- **Exécution linéaire** : Collecte → Sérialisation → Envoi → Fin
- **Retry intelligent** : 3 tentatives avec délai configurable
- **Gestion d'erreurs robuste** : Validation et reporting d'erreurs détaillé

### Logique mutualisée

**Fonctions communes dans `utils` :**
- **`process_session_event()`** : Logique partagée entre logon.exe et logout.exe
- **`process_hardware_info()`** : Traitement spécialisé pour matos.exe
- **Validation centralisée** : Contrôles de données avant envoi
- **Gestion d'erreurs unifiée** : Messages et codes de retour cohérents

**Avantages :**
- **Code réduit** : Binaires de ~10 lignes au lieu de ~50
- **Maintenance simplifiée** : Modifications dans un seul endroit
- **Cohérence** : Comportement identique entre les binaires
- **Performance** : Pas de duplication de code

## Dépendances

- `sysinfo` (0.37.2) : Récupération des informations système
- `minreq` (2.14) : Client HTTP synchrone léger
- `serde` + `serde_json` (1.0) : Sérialisation JSON
- `chrono` (0.4) : Gestion des timestamps
- `whoami` (1.4) : Détection du nom d'utilisateur

## Traitement des données

Les données sont envoyées au serveur de monitoring via HTTP POST avec un payload JSON. Le serveur traite les informations reçues et les stocke dans une **base de données SQLite** pour analyse et requêtes dynamiques.

### Stockage SQLite
- **Base de données** : `/var/lib/winlog/winlog.db`
- **Table principale** : `events` avec index optimisés
- **Concurrence** : Mode WAL pour écritures simultanées
- **Performance** : Support de centaines de connexions simultanées

### Scripts de gestion
- `php/creation_base.php` : Création de la base et des tables
- `php/delete_base.php` : Suppression complète de la base
- `php/purge_base.php` : Vidage des données (conserve la structure)
- `serde` + `serde_json` (1.0) : Sérialisation JSON
- `chrono` (0.4) : Gestion des timestamps
- `whoami` (1.4) : Récupération du nom d'utilisateur

## Format des données envoyées

```json
{
  "username": "jerome.win11_jb",
  "action": "C",
  "timestamp": "2026-01-12T14:30:15Z",
  "hostname": "WIN11-JB",
  "os_info": {
    "os_name": "Windows",
    "os_version": "11",
    "kernel_version": "10.0.22631"
  },
  "hardware_info": {
    "cpu_count": 12,
    "cpu_brand": "Intel Core i7-12700K",
    "memory_total": 33554432
  }
}
```

**Codes d'action :**
- **"C"** : Connexion (ouverture de session)
- **"D"** : Déconnexion (fermeture de session)
- **"M"** : Matériel (collecte d'informations hardware)

**Optimisations JSON :**
- Codes courts pour réduire la bande passante
- Structure cohérente entre tous les types d'événements
- Timestamps UTC au format ISO 8601

## Performances et optimisations

**Caractéristiques techniques :**
- **Binaires légers** : ~1MB chaque binaire
- **Démarrage rapide** : ~10ms de lancement
- **Empreinte mémoire** : <5MB pendant l'exécution
- **Client HTTP** : `minreq` pour des requêtes synchrones sans dépendances lourdes
- **Pas de runtime** : Architecture synchrone pure, pas d'overhead async

**Optimisé pour Windows GPO :**
- Scripts d'ouverture/fermeture de session ultra-rapides
- Exécution one-shot sans résidu mémoire
- Gestion d'erreurs silencieuse avec retry automatique

## Déploiement

Les binaires seront intégrés dans les scripts de politique de groupe Windows pour être exécutés automatiquement :
- `logon.exe` : Script d'ouverture de session
- `logout.exe` : Script de fermeture de session
- `matos.exe` : Tâche planifiée ou exécution manuelle
