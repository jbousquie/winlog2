#!/bin/bash
###############################################################################
# Script de migration de l'ancienne structure vers la nouvelle
# Transforme la table unique 'events' en 'events_today' + 'events_history'
#
# Usage: ./migrate_to_new_structure.sh
###############################################################################

set -e

# Configuration
DB_PATH="/var/www/ferron/winlog/data/winlog.db"
BACKUP_PATH="/var/www/ferron/winlog/data/winlog_backup_$(date +%Y%m%d_%H%M%S).db"

echo "=== Migration vers la nouvelle structure partitionnée ==="
echo ""

# Vérifier que la base existe
if [ ! -f "$DB_PATH" ]; then
    echo "❌ Base de données introuvable : $DB_PATH"
    echo "Rien à migrer. Utilisez ./create_base.sh pour créer une nouvelle base."
    exit 1
fi

# Vérifier sqlite3
if ! command -v sqlite3 &> /dev/null; then
    echo "❌ sqlite3 n'est pas installé"
    exit 1
fi

echo "✓ Base existante détectée : $DB_PATH"

# Vérifier si l'ancienne structure existe
HAS_OLD=$(sqlite3 "$DB_PATH" "SELECT name FROM sqlite_master WHERE type='table' AND name='events';" 2>/dev/null || echo "")

if [ -z "$HAS_OLD" ]; then
    echo "❌ Table 'events' introuvable"
    echo "   La base semble déjà migrée ou corrompue"
    exit 1
fi

echo "✓ Table 'events' détectée (ancienne structure)"

# Vérifier si la nouvelle structure existe déjà
HAS_NEW=$(sqlite3 "$DB_PATH" "SELECT name FROM sqlite_master WHERE type='table' AND name='events_today';" 2>/dev/null || echo "")

if [ -n "$HAS_NEW" ]; then
    echo "⚠ La nouvelle structure existe déjà !"
    echo "   La migration semble déjà effectuée."
    exit 1
fi

# Statistiques avant migration
TOTAL_EVENTS=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM events")
echo ""
echo "Statistiques actuelles :"
echo "  - Événements dans 'events' : $TOTAL_EVENTS"

# Répartition par action
echo ""
echo "Répartition par action :"
sqlite3 "$DB_PATH" "
    SELECT '  ' || 
           CASE action 
               WHEN 'C' THEN 'Connexions (C)'
               WHEN 'D' THEN 'Déconnexions (D)'
               WHEN 'M' THEN 'Matériel (M)'
               ELSE 'Autre'
           END || ' : ' || COUNT(*)
    FROM events 
    GROUP BY action
    ORDER BY action
"

# Taille actuelle
FILE_SIZE=$(du -h "$DB_PATH" | cut -f1)
echo ""
echo "Taille actuelle : $FILE_SIZE"

# Demander confirmation
echo ""
echo "⚠ Cette opération va :"
echo "  1. Créer un backup de la base actuelle"
echo "  2. Créer les nouvelles tables (events_today, events_history)"
echo "  3. Migrer les données du jour vers events_today"
echo "  4. Migrer les données anciennes vers events_history"
echo "  5. Renommer l'ancienne table events en events_old (conservée)"
echo ""
echo "Confirmez la migration en tapant 'MIGRER' : "
read -r CONFIRMATION

if [ "$CONFIRMATION" != "MIGRER" ]; then
    echo "❌ Migration annulée."
    exit 0
fi

# Créer le backup
echo ""
echo "Création du backup..."
cp "$DB_PATH" "$BACKUP_PATH"
echo "✓ Backup créé : $BACKUP_PATH"

# Effectuer la migration
echo ""
echo "Migration en cours..."

sqlite3 "$DB_PATH" <<'EOF'
BEGIN TRANSACTION;

-- ============================================================================
-- Créer la nouvelle structure
-- ============================================================================

-- Table events_today (données du jour)
CREATE TABLE IF NOT EXISTS events_today (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username VARCHAR(50) NOT NULL,
    action CHAR(1) NOT NULL CHECK (action IN ('C', 'D', 'M')),
    timestamp DATETIME NOT NULL,
    hostname VARCHAR(100),
    source_ip VARCHAR(45),
    server_timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    os_name VARCHAR(50),
    os_version VARCHAR(100),
    kernel_version VARCHAR(50),
    hardware_info TEXT,
    session_uuid VARCHAR(100),
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Table events_history (archive)
CREATE TABLE IF NOT EXISTS events_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username VARCHAR(50) NOT NULL,
    action CHAR(1) NOT NULL CHECK (action IN ('C', 'D', 'M')),
    timestamp DATETIME NOT NULL,
    hostname VARCHAR(100),
    source_ip VARCHAR(45),
    server_timestamp DATETIME,
    os_name VARCHAR(50),
    os_version VARCHAR(100),
    kernel_version VARCHAR(50),
    hardware_info TEXT,
    session_uuid VARCHAR(100),
    created_at DATETIME,
    archived_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================================
-- Migrer les données
-- ============================================================================

-- Migrer les données du jour vers events_today
INSERT INTO events_today (
    username, action, timestamp, hostname, source_ip, server_timestamp,
    os_name, os_version, kernel_version, hardware_info, session_uuid, created_at
)
SELECT 
    username, action, timestamp, hostname, source_ip, server_timestamp,
    os_name, os_version, kernel_version, hardware_info, session_uuid, created_at
FROM events
WHERE DATE(timestamp) = DATE('now');

-- Migrer les données anciennes vers events_history
INSERT INTO events_history (
    username, action, timestamp, hostname, source_ip, server_timestamp,
    os_name, os_version, kernel_version, hardware_info, session_uuid, created_at
)
SELECT 
    username, action, timestamp, hostname, source_ip, server_timestamp,
    os_name, os_version, kernel_version, hardware_info, session_uuid, created_at
FROM events
WHERE DATE(timestamp) < DATE('now');

-- ============================================================================
-- Créer les index
-- ============================================================================

-- Index events_today
CREATE INDEX IF NOT EXISTS idx_today_username_action ON events_today(username, action);
CREATE INDEX IF NOT EXISTS idx_today_timestamp ON events_today(timestamp);
CREATE INDEX IF NOT EXISTS idx_today_hostname ON events_today(hostname);
CREATE INDEX IF NOT EXISTS idx_today_session ON events_today(session_uuid);
CREATE INDEX IF NOT EXISTS idx_today_action_time ON events_today(action, timestamp);

-- Index events_history
CREATE INDEX IF NOT EXISTS idx_history_username ON events_history(username);
CREATE INDEX IF NOT EXISTS idx_history_timestamp ON events_history(timestamp);
CREATE INDEX IF NOT EXISTS idx_history_hostname ON events_history(hostname);
CREATE INDEX IF NOT EXISTS idx_history_session ON events_history(session_uuid);
CREATE INDEX IF NOT EXISTS idx_history_date ON events_history(DATE(timestamp));

-- ============================================================================
-- Créer la vue combinée
-- ============================================================================

CREATE VIEW IF NOT EXISTS events_all AS
    SELECT id, username, action, timestamp, hostname, source_ip, 
           server_timestamp, os_name, os_version, kernel_version,
           hardware_info, session_uuid, created_at, 'today' as source
    FROM events_today
    UNION ALL
    SELECT id, username, action, timestamp, hostname, source_ip,
           server_timestamp, os_name, os_version, kernel_version,
           hardware_info, session_uuid, created_at, 'history' as source
    FROM events_history;

-- ============================================================================
-- Renommer l'ancienne table (conservation pour sécurité)
-- ============================================================================

ALTER TABLE events RENAME TO events_old;

COMMIT;
EOF

if [ $? -eq 0 ]; then
    echo "✓ Migration effectuée avec succès"
    
    # Statistiques après migration
    TODAY_COUNT=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM events_today")
    HISTORY_COUNT=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM events_history")
    OLD_COUNT=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM events_old")
    
    echo ""
    echo "=== Migration terminée ==="
    echo "Statistiques après migration :"
    echo "  - events_today : $TODAY_COUNT enregistrements"
    echo "  - events_history : $HISTORY_COUNT enregistrements"
    echo "  - events_old (backup) : $OLD_COUNT enregistrements"
    echo "  - TOTAL migré : $((TODAY_COUNT + HISTORY_COUNT)) / $OLD_COUNT"
    
    # Vérification
    if [ "$((TODAY_COUNT + HISTORY_COUNT))" -eq "$OLD_COUNT" ]; then
        echo ""
        echo "✓ Vérification : Toutes les données ont été migrées correctement"
    else
        echo ""
        echo "⚠ ATTENTION : Nombre d'enregistrements différent !"
        echo "   Vérifiez manuellement avant de supprimer events_old"
    fi
    
    # Optimiser la base
    echo ""
    echo "Optimisation de la base (VACUUM)..."
    sqlite3 "$DB_PATH" "VACUUM;"
    echo "✓ Base optimisée"
    
    # Taille finale
    NEW_FILE_SIZE=$(du -h "$DB_PATH" | cut -f1)
    echo ""
    echo "Taille après migration : $NEW_FILE_SIZE (avant: $FILE_SIZE)"
    
    # Afficher la structure finale
    echo ""
    echo "Structure finale de la base :"
    sqlite3 "$DB_PATH" ".tables"
    
    echo ""
    echo "✅ Migration terminée avec succès !"
    echo ""
    echo "Prochaines étapes :"
    echo "  1. Tester les scripts PHP avec la nouvelle structure"
    echo "  2. Vérifier que tout fonctionne correctement"
    echo "  3. Supprimer events_old si tout est OK :"
    echo "     sqlite3 $DB_PATH 'DROP TABLE events_old;'"
    echo "  4. Configurer la rotation quotidienne : crontab -e"
    echo "     0 1 * * * $(realpath ./rotate_daily.sh)"
    
else
    echo "❌ Erreur lors de la migration"
    echo "   Le backup est disponible : $BACKUP_PATH"
    echo "   Restaurez avec : cp $BACKUP_PATH $DB_PATH"
    exit 1
fi
