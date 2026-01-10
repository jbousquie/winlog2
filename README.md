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

- **Client HTTP** : Gestion des requêtes POST vers le serveur de monitoring
- **Collecte système** : Récupération des informations (username, hostname, OS, matériel)
- **Sérialisation** : Structures de données JSON pour l'envoi
- **Utilitaires** : Formatage timestamps, gestion d'erreurs, logging

## Dépendances

- `sysinfo` (0.37.2) : Récupération des informations système
- `reqwest` (0.11) : Client HTTP avec support JSON
- `tokio` (1.0) : Runtime asynchrone requis par reqwest

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

## Déploiement

Les binaires seront intégrés dans les scripts de politique de groupe Windows pour être exécutés automatiquement :
- `logon.exe` : Script d'ouverture de session
- `logout.exe` : Script de fermeture de session
- `matos.exe` : Tâche planifiée ou exécution manuelle
