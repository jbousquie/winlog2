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
- **Action** : "login"
- **Données collectées** : Username, timestamp, informations système de base
- **Comportement** : Envoi immédiat des données au serveur

#### `logout.exe`
- **Rôle** : Exécuté lors de la fermeture de session Windows
- **Action** : "logout"
- **Données collectées** : Username, timestamp, durée de session
- **Comportement** : Envoi immédiat des données au serveur

#### `matos.exe`
- **Rôle** : Collecte d'informations détaillées sur le matériel
- **Action** : "hardware_info"
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
- **Sérialisation JSON** : Structures de données sérialisables automatiquement
- **Utilitaires** : Formatage timestamps, validation des données

### Architecture synchrone optimisée

Les binaires utilisent une architecture **100% synchrone** optimisée pour des scripts d'exécution rapide :
- **Démarrage instantané** : Pas d'overhead de runtime asynchrone
- **Exécution linéaire** : Collecte → Sérialisation → Envoi → Fin
- **Retry intelligent** : 3 tentatives avec délai configurable
- **Gestion d'erreurs robuste** : Validation et reporting d'erreurs détaillé

## Dépendances

- `sysinfo` (0.37.2) : Récupération des informations système
- `minreq` (2.14) : Client HTTP synchrone léger
- `serde` + `serde_json` (1.0) : Sérialisation JSON
- `chrono` (0.4) : Gestion des timestamps
- `whoami` (1.4) : Récupération du nom d'utilisateur

## Format des données envoyées

```json
{
  "username": "jerome.win11_jb",
  "action": "login|logout|hardware_info",
  "timestamp": "2026-01-10T14:30:15Z",
  "hostname": "WIN11-JB",
  "os_info": {
    "name": "Windows",
    "version": "11",
    "architecture": "x86_64"
  },
  "hardware_info": {
    "cpu": "Intel Core i7-12700K",
    "memory_total": 32768,
    "disk_usage": [...]
  }
}
```

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
