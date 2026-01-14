#!/bin/bash
###############################################################################
# Script de suppression de la base SQLite Winlog
# Supprime complètement le fichier de base de données et ses fichiers WAL/SHM
#
# Usage: ./delete_base.sh
# ATTENTION: Cette opération est irréversible !
###############################################################################

set -e

# Configuration
# Chemin relatif au répertoire du projet
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
DB_PATH="$PROJECT_DIR/data/winlog.db"

echo "=== Suppression de la base Winlog ==="
echo ""
echo "⚠ ATTENTION : Cette opération va supprimer DÉFINITIVEMENT la base !"
echo "Base à supprimer : $DB_PATH"
echo ""

# Vérifier si la base existe
if [ ! -f "$DB_PATH" ]; then
    echo "ℹ Base de données inexistante : $DB_PATH"
    echo "Rien à supprimer."
    exit 0
fi

# Afficher les informations avant suppression
FILE_SIZE=$(du -h "$DB_PATH" | cut -f1)
echo "Taille actuelle : $FILE_SIZE"

# Compter les enregistrements si possible
if command -v sqlite3 &> /dev/null; then
    echo ""
    echo "Statistiques :"
    
    # Compter events_today
    TODAY_COUNT=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM events_today" 2>/dev/null || echo "0")
    echo "  - events_today : $TODAY_COUNT enregistrements"
    
    # Compter events_history
    HISTORY_COUNT=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM events_history" 2>/dev/null || echo "0")
    echo "  - events_history : $HISTORY_COUNT enregistrements"
    
    TOTAL_COUNT=$((TODAY_COUNT + HISTORY_COUNT))
    echo "  - TOTAL : $TOTAL_COUNT enregistrements"
else
    echo "⚠ sqlite3 non disponible - impossible de compter les enregistrements"
fi

# Demander confirmation
echo ""
echo "Confirmez la suppression en tapant 'SUPPRIMER' : "
read -r CONFIRMATION

if [ "$CONFIRMATION" != "SUPPRIMER" ]; then
    echo "❌ Suppression annulée."
    exit 0
fi

echo ""
echo "Suppression en cours..."

# Supprimer le fichier principal
if rm -f "$DB_PATH"; then
    echo "✓ Base de données supprimée"
else
    echo "❌ Erreur lors de la suppression du fichier principal"
    exit 1
fi

# Supprimer les fichiers WAL et SHM (mode WAL)
WAL_FILE="${DB_PATH}-wal"
SHM_FILE="${DB_PATH}-shm"

if [ -f "$WAL_FILE" ]; then
    rm -f "$WAL_FILE"
    echo "✓ Fichier WAL supprimé"
fi

if [ -f "$SHM_FILE" ]; then
    rm -f "$SHM_FILE"
    echo "✓ Fichier SHM supprimé"
fi

echo ""
echo "=== Suppression terminée ==="
echo "Utilisez ./create_base.sh pour recréer la base."
