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

### 1. create_base.sh
**Création de la base avec la structure partitionnée (installation neuve)**

```bash
./create_base.sh
```

**Actions effectuées :**
- Vérification des prérequis (sqlite3 installé)
- Création du répertoire `serveur/data/` si nécessaire
- Création des tables `events_today` et `events_history`
- Création de 11 index optimisés
- Création de la vue `events_all`
- Configuration SQLite (mode WAL, synchronous NORMAL, busy_timeout 30s)

**Sortie :**
```
=== Création de la base Winlog (structure partitionnée) ===
✓ SQLite3 disponible (3.37.2)
✓ Utilisateur : www-data
✓ Répertoire existant : serveur/data
✓ Tables créées : events_today, events_history
✓ Vue créée : events_all
✓ Index créés (11 au total)

=== Création terminée avec succès ===
Base de données : serveur/data/winlog.db
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
- Log toutes les opérations dans `serveur/data/rotation.log`

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
tail -f serveur/data/rotation.log
```

---

## Workflow complet

### Installation neuve

```bash
cd /home/jerome/scripts/rust/winlog2/serveur/scripts

# 1. Créer la base
./create_base.sh

# 2. Configurer le cron pour rotation automatique
crontab -e
# Ajouter : 0 1 * * * /path/to/rotate_daily.sh
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

## Requêtes SQL utiles

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

## Performances

| Opération | Performance |
|-----------|-------------|
| SELECT sessions actives | ~5ms (100 lignes) |
| INSERT événement | ~5ms |
| Recherche historique | ~80ms (via events_all) |

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
lsof serveur/data/winlog.db

# Forcer la fermeture du mode WAL
sqlite3 serveur/data/winlog.db "PRAGMA journal_mode=DELETE;"
sqlite3 serveur/data/winlog.db "PRAGMA journal_mode=WAL;"
```

### Permissions insuffisantes
```bash
# Donner les permissions appropriées
sudo chown -R $USER:$USER serveur/data/
sudo chmod 755 serveur/data/
sudo chmod 664 serveur/data/winlog.db*
```

### Vérifier l'intégrité
```bash
sqlite3 serveur/data/winlog.db "PRAGMA integrity_check;"
```

---

## Notes importantes

1. **Rotation quotidienne obligatoire** : Sans rotation, `events_today` grossit indéfiniment et perd son avantage performance
2. **Backup réguliers** : Sauvegarder la base régulièrement (rotation crée des backups automatiques)
3. **Mode WAL** : Permet les lectures concurrentes pendant les écritures (essentiel pour la performance)

---

## Prochaine étape : Migration vers Rust

Le schéma SQLite est compatible avec le serveur Rust (Axum + SQLx) actuel.
