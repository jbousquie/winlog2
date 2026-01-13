# Scripts de gestion de la base de données Winlog

Ce répertoire contient les scripts bash pour la gestion de la base de données SQLite avec la nouvelle structure partitionnée (events_today + events_history).

## Architecture de la base

### Structure à deux tables

```
┌─────────────────────────────────────┐
│   events_today                      │  ← Données du jour (INSERT/SELECT fréquents)
│   - Connexions actives              │
│   - Index optimisés                 │
│   - Mode WAL                        │
└─────────────────────────────────────┘
           │ Rotation quotidienne (cron 01:00)
           ↓
┌─────────────────────────────────────┐
│   events_history                    │  ← Archive (SELECT occasionnels)
│   - Toutes connexions passées       │
│   - Index sur date + username       │
└─────────────────────────────────────┘
```

### Vue combinée

La vue `events_all` permet d'interroger les deux tables simultanément :
```sql
SELECT * FROM events_all WHERE username='jerome';
```

## Scripts disponibles

### 0. migrate_to_new_structure.sh ⭐
**Migration depuis l'ancienne structure (table unique 'events')**

```bash
./migrate_to_new_structure.sh
```

**⚠️ À utiliser UNIQUEMENT si vous avez déjà une base avec l'ancienne structure !**

**Actions effectuées :**
- Crée un backup automatique avec timestamp
- Crée les nouvelles tables `events_today` et `events_history`
- Migre les données du jour vers `events_today`
- Migre les données anciennes vers `events_history`
- Crée tous les index optimisés
- Crée la vue `events_all`
- Renomme l'ancienne table en `events_old` (conservée pour sécurité)

**Sortie typique :**
```
=== Migration vers la nouvelle structure partitionnée ===
✓ Backup créé : /var/www/ferron/winlog/data/winlog_backup_20260113_120230.db
✓ Migration effectuée avec succès

Statistiques après migration :
  - events_today : 42 enregistrements
  - events_history : 358 enregistrements
  - events_old (backup) : 400 enregistrements
  - TOTAL migré : 400 / 400

✓ Vérification : Toutes les données ont été migrées correctement
```

**Nettoyage post-migration :**
```bash
# Après avoir vérifié que tout fonctionne
sqlite3 /var/www/ferron/winlog/data/winlog.db 'DROP TABLE events_old;'
```

---

### 1. create_base.sh
**Création de la base avec la nouvelle structure (installation neuve)**

```bash
./create_base.sh
```

**Actions effectuées :**
- Vérification des prérequis (sqlite3 installé)
- Création du répertoire `/var/www/ferron/winlog/data/` si nécessaire
- Création des tables `events_today` et `events_history`
- Création de 11 index optimisés
- Création de la vue `events_all`
- Configuration SQLite (mode WAL, synchronous NORMAL, busy_timeout 30s)

**Sortie :**
```
=== Création de la base Winlog (structure partitionnée) ===
✓ SQLite3 disponible (3.37.2)
✓ Utilisateur : www-data
✓ Répertoire existant : /var/www/ferron/winlog/data
✓ Tables créées : events_today, events_history
✓ Vue créée : events_all
✓ Index créés (11 au total)

=== Création terminée avec succès ===
Base de données : /var/www/ferron/winlog/data/winlog.db
Taille : 32K
```

---

### 2. delete_base.sh
**Suppression complète de la base**

```bash
./delete_base.sh
```

**Actions effectuées :**
- Affiche les statistiques avant suppression
- Demande confirmation (`SUPPRIMER`)
- Supprime le fichier principal `.db`
- Supprime les fichiers WAL et SHM

**⚠️ ATTENTION : Opération irréversible !**

---

### 3. purge_base.sh
**Vidage des données (conserve la structure)**

```bash
# Vider les deux tables
./purge_base.sh --all

# Vider uniquement events_today
./purge_base.sh --today

# Vider uniquement events_history
./purge_base.sh --history
```

**Actions effectuées :**
- Affiche les statistiques (nombre d'enregistrements, répartition par action)
- Demande confirmation (`VIDER`, `VIDER_TODAY`, ou `VIDER_HISTORY`)
- Supprime les données de la/des table(s) ciblée(s)
- Réinitialise l'auto-increment
- Exécute VACUUM pour récupérer l'espace disque

**Utilisation typique :**
```bash
# Vider uniquement l'historique ancien (garder le jour courant)
./purge_base.sh --history

# Tout vider pour un nouveau départ
./purge_base.sh --all
```

---

### 4. rotate_daily.sh
**Rotation quotidienne automatique**

```bash
./rotate_daily.sh
```

**Actions effectuées :**
- Déplace toutes les données de `events_today` vers `events_history`
- Vide `events_today`
- Réinitialise l'auto-increment
- Exécute VACUUM pour optimiser
- Log toutes les opérations dans `/var/www/ferron/winlog/data/rotation.log`

**Configuration cron (recommandé) :**
```bash
# Rotation quotidienne à 01:00
0 1 * * * /home/jerome/scripts/rust/winlog2/serveur/scripts/rotate_daily.sh

# Avec redirection complète des logs
0 1 * * * /home/jerome/scripts/rust/winlog2/serveur/scripts/rotate_daily.sh >> /var/log/winlog_rotation.log 2>&1
```

**Installation cron :**
```bash
# Éditer le crontab
crontab -e

# Ajouter la ligne
0 1 * * * /path/to/rotate_daily.sh
```

**Vérification des logs :**
```bash
tail -f /var/www/ferron/winlog/data/rotation.log
```

---

## Workflow complet

### Migration depuis ancienne structure

```bash
cd /home/jerome/scripts/rust/winlog2/serveur/scripts

# 1. Migrer la base existante
./migrate_to_new_structure.sh

# 2. Adapter index.php (voir section "Migration PHP")
# Remplacer toutes les requêtes 'events' par 'events_today'

# 3. Tester que tout fonctionne

# 4. Nettoyer l'ancienne table
sqlite3 /var/www/ferron/winlog/data/winlog.db 'DROP TABLE events_old;'

# 5. Configurer le cron pour rotation automatique
crontab -e
# Ajouter : 0 1 * * * /path/to/rotate_daily.sh
```

### Installation neuve (sans données existantes)

```bash
cd /home/jerome/scripts/rust/winlog2/serveur/scripts

# 1. Créer la base
./create_base.sh

# 2. Configurer le cron pour rotation automatique
crontab -e
# Ajouter : 0 1 * * * /path/to/rotate_daily.sh

# 3. Adapter index.php pour utiliser events_today
# (voir section "Migration PHP" ci-dessous)
```

### Maintenance

```bash
# Vider uniquement l'historique (garder aujourd'hui)
./purge_base.sh --history

# Forcer une rotation manuelle
./rotate_daily.sh

# Recréer complètement (⚠️ perte de données)
./delete_base.sh
./create_base.sh
```

---

## Migration du code PHP

### Modifications requises dans index.php

**Avant (ancienne structure) :**
```php
$stmt = $pdo->prepare("INSERT INTO events (...) VALUES (...)");
$openSession = $pdo->query("SELECT * FROM events WHERE action='C' ...");
```

**Après (nouvelle structure) :**
```php
// Toutes les insertions vont dans events_today
$stmt = $pdo->prepare("INSERT INTO events_today (...) VALUES (...)");

// Recherche de sessions ouvertes (seulement dans today)
$openSession = $pdo->query("SELECT * FROM events_today WHERE action='C' ...");

// Requêtes globales (today + history)
$allSessions = $pdo->query("SELECT * FROM events_all WHERE username=?");
```

### Requêtes SQL courantes

```sql
-- Sessions actives en ce moment (rapide)
SELECT * FROM events_today 
WHERE action='C' 
AND NOT EXISTS (
    SELECT 1 FROM events_today e2 
    WHERE e2.session_uuid = events_today.session_uuid 
    AND e2.action='D'
);

-- Historique d'un utilisateur (toutes données)
SELECT * FROM events_all 
WHERE username='jerome' 
ORDER BY timestamp DESC;

-- Recherche dans l'historique uniquement
SELECT * FROM events_history 
WHERE DATE(timestamp) = '2026-01-12';
```

---

## Performances attendues

| Opération | events (ancienne) | events_today (nouvelle) | Gain |
|-----------|-------------------|-------------------------|------|
| SELECT sessions actives | ~50ms (10k lignes) | ~5ms (100 lignes) | **10x** |
| INSERT événement | ~10ms | ~5ms | **2x** |
| Recherche historique | ~100ms | ~80ms (via events_all) | **1.2x** |

**Avantages :**
- ✅ Table `events_today` toujours petite (~100-1000 lignes max)
- ✅ Pas de lock prolongé sur les lectures de sessions actives
- ✅ Historique interrogeable via `events_all` ou `events_history`
- ✅ Rotation automatique transparente

---

## Prérequis

```bash
# Installer SQLite3
sudo apt install sqlite3

# Vérifier la version
sqlite3 --version
# Minimum requis : 3.7.0 (pour mode WAL)
```

---

## Dépannage

### Erreur "database is locked"
```bash
# Vérifier les processus utilisant la base
lsof /var/www/ferron/winlog/data/winlog.db

# Forcer la fermeture du mode WAL
sqlite3 /var/www/ferron/winlog/data/winlog.db "PRAGMA journal_mode=DELETE;"
sqlite3 /var/www/ferron/winlog/data/winlog.db "PRAGMA journal_mode=WAL;"
```

### Permissions insuffisantes
```bash
# Donner les permissions à www-data (Apache/PHP)
sudo chown -R www-data:www-data /var/www/ferron/winlog/data/
sudo chmod 755 /var/www/ferron/winlog/data/
sudo chmod 664 /var/www/ferron/winlog/data/winlog.db*
```

### Vérifier l'intégrité
```bash
sqlite3 /var/www/ferron/winlog/data/winlog.db "PRAGMA integrity_check;"
```

---

## Notes importantes

1. **Rotation quotidienne obligatoire** : Sans rotation, `events_today` grossit indéfiniment et perd son avantage performance
2. **Backup avant migration** : Sauvegarder l'ancienne base avant d'exécuter `create_base.sh`
3. **Compatibilité PHP** : Les scripts PHP existants doivent être adaptés pour utiliser `events_today`
4. **Mode WAL** : Permet les lectures concurrentes pendant les écritures (essentiel pour la performance)

---

## Prochaine étape : Migration vers Rust

Ces scripts préparent la structure pour la future migration du serveur PHP vers Rust (Axum + SQLx).
Le schéma SQLite restera identique, seul le code du serveur HTTP changera.
