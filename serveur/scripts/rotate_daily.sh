#!/bin/bash
###############################################################################
# Script de rotation quotidienne des données Winlog
# Déplace les données de events_today vers events_history
# À exécuter automatiquement chaque nuit (cron : 0 1 * * *)
#
# Usage: ./rotate_daily.sh
###############################################################################

set -e

# Configuration
# Chemin relatif au répertoire du projet
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
DB_DIR="$PROJECT_DIR/data"
DB_PATH="$DB_DIR/winlog.db"
LOG_FILE="$DB_DIR/rotation.log"

# Fonction de log avec timestamp
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*" | tee -a "$LOG_FILE"
}

log "=== Début de la rotation quotidienne ==="

# Vérifier que la base existe
if [ ! -f "$DB_PATH" ]; then
    log "❌ Base de données introuvable : $DB_PATH"
    exit 1
fi

# Vérifier sqlite3
if ! command -v sqlite3 &> /dev/null; then
    log "❌ sqlite3 n'est pas installé"
    exit 1
fi

# Compter les enregistrements avant rotation
BEFORE_TODAY=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM events_today")
BEFORE_HISTORY=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM events_history")

log "État avant rotation :"
log "  - events_today : $BEFORE_TODAY enregistrements"
log "  - events_history : $BEFORE_HISTORY enregistrements"

if [ "$BEFORE_TODAY" -eq 0 ]; then
    log "ℹ Aucune donnée à archiver dans events_today"
    log "=== Rotation terminée (rien à faire) ==="
    exit 0
fi

# Effectuer la rotation dans une transaction
log "Déplacement des données vers l'historique..."

sqlite3 "$DB_PATH" <<'EOF'
BEGIN TRANSACTION;

-- Copier toutes les données de events_today vers events_history
INSERT INTO events_history (
    username, action, timestamp, hostname, source_ip, server_timestamp,
    os_name, os_version, kernel_version, hardware_info, session_uuid, created_at
)
SELECT 
    username, action, timestamp, hostname, source_ip, server_timestamp,
    os_name, os_version, kernel_version, hardware_info, session_uuid, created_at
FROM events_today;

-- Vider events_today
DELETE FROM events_today;

-- Réinitialiser l'auto-increment
DELETE FROM sqlite_sequence WHERE name='events_today';

COMMIT;
EOF

if [ $? -eq 0 ]; then
    log "✓ Données déplacées avec succès"
    
    # Compter après rotation
    AFTER_TODAY=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM events_today")
    AFTER_HISTORY=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM events_history")
    ARCHIVED=$((AFTER_HISTORY - BEFORE_HISTORY))
    
    log "État après rotation :"
    log "  - events_today : $AFTER_TODAY enregistrements"
    log "  - events_history : $AFTER_HISTORY enregistrements"
    log "  - Archivés : $ARCHIVED enregistrements"
    
    # Optimiser la base (récupérer l'espace)
    log "Optimisation de la base (VACUUM)..."
    sqlite3 "$DB_PATH" "VACUUM;"
    
    log "✓ Base optimisée"
    
    # Taille finale
    FILE_SIZE=$(du -h "$DB_PATH" | cut -f1)
    log "Taille de la base : $FILE_SIZE"
    
    log "=== Rotation terminée avec succès ==="
else
    log "❌ Erreur lors de la rotation"
    exit 1
fi
