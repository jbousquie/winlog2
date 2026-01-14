#!/bin/bash
###############################################################################
# Script de vidage de la base SQLite Winlog
# Supprime toutes les données des deux tables mais conserve la structure
#
# Usage: ./purge_base.sh [--today|--history|--all]
# Options:
#   --today   : Vide uniquement events_today
#   --history : Vide uniquement events_history
#   --all     : Vide les deux tables (défaut)
###############################################################################

set -e

# Configuration
# Chemin relatif au répertoire du projet
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
DB_PATH="$PROJECT_DIR/data/winlog.db"

# Déterminer la cible du vidage
TARGET="all"
if [ "$1" = "--today" ]; then
    TARGET="today"
elif [ "$1" = "--history" ]; then
    TARGET="history"
elif [ "$1" = "--all" ]; then
    TARGET="all"
elif [ -n "$1" ]; then
    echo "❌ Option invalide : $1"
    echo "Usage : $0 [--today|--history|--all]"
    exit 1
fi

echo "=== Vidage de la base Winlog ==="
echo ""

case $TARGET in
    today)
        echo "Cible : events_today uniquement"
        ;;
    history)
        echo "Cible : events_history uniquement"
        ;;
    all)
        echo "Cible : Toutes les tables (events_today + events_history)"
        ;;
esac

echo "⚠ ATTENTION : Cette opération va supprimer les données !"
echo "La structure de la base sera conservée."
echo "Base : $DB_PATH"
echo ""

# Vérifier si la base existe
if [ ! -f "$DB_PATH" ]; then
    echo "❌ Base de données introuvable : $DB_PATH"
    echo "Utilisez ./create_base.sh pour créer la base."
    exit 1
fi

# Vérifier que sqlite3 est disponible
if ! command -v sqlite3 &> /dev/null; then
    echo "❌ sqlite3 n'est pas installé"
    exit 1
fi

echo "✓ Connexion établie"
echo ""

# Afficher les statistiques avant vidage
echo "Statistiques actuelles :"

TODAY_COUNT=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM events_today" 2>/dev/null || echo "0")
HISTORY_COUNT=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM events_history" 2>/dev/null || echo "0")
TOTAL_COUNT=$((TODAY_COUNT + HISTORY_COUNT))

echo "  - events_today : $TODAY_COUNT enregistrements"
echo "  - events_history : $HISTORY_COUNT enregistrements"
echo "  - TOTAL : $TOTAL_COUNT enregistrements"

# Vérifier si déjà vide
if [ "$TARGET" = "all" ] && [ "$TOTAL_COUNT" -eq 0 ]; then
    echo ""
    echo "ℹ La base est déjà vide."
    exit 0
elif [ "$TARGET" = "today" ] && [ "$TODAY_COUNT" -eq 0 ]; then
    echo ""
    echo "ℹ La table events_today est déjà vide."
    exit 0
elif [ "$TARGET" = "history" ] && [ "$HISTORY_COUNT" -eq 0 ]; then
    echo ""
    echo "ℹ La table events_history est déjà vide."
    exit 0
fi

# Répartition par type d'action
echo ""
echo "Répartition par type d'action :"

if [ "$TARGET" = "today" ] || [ "$TARGET" = "all" ]; then
    echo "  events_today :"
    sqlite3 "$DB_PATH" "
        SELECT '    ' || 
               CASE action 
                   WHEN 'C' THEN 'Connexions (C)'
                   WHEN 'D' THEN 'Déconnexions (D)'
                   WHEN 'M' THEN 'Matériel (M)'
                   ELSE 'Autre'
               END || ' : ' || COUNT(*)
        FROM events_today 
        GROUP BY action
        ORDER BY action
    " 2>/dev/null || echo "    (aucune donnée)"
fi

if [ "$TARGET" = "history" ] || [ "$TARGET" = "all" ]; then
    echo "  events_history :"
    sqlite3 "$DB_PATH" "
        SELECT '    ' || 
               CASE action 
                   WHEN 'C' THEN 'Connexions (C)'
                   WHEN 'D' THEN 'Déconnexions (D)'
                   WHEN 'M' THEN 'Matériel (M)'
                   ELSE 'Autre'
               END || ' : ' || COUNT(*)
        FROM events_history 
        GROUP BY action
        ORDER BY action
    " 2>/dev/null || echo "    (aucune donnée)"
fi

# Taille du fichier
FILE_SIZE=$(du -h "$DB_PATH" | cut -f1)
echo ""
echo "Taille actuelle de la base : $FILE_SIZE"

# Demander confirmation
echo ""
case $TARGET in
    today)
        CONFIRM_WORD="VIDER_TODAY"
        ;;
    history)
        CONFIRM_WORD="VIDER_HISTORY"
        ;;
    all)
        CONFIRM_WORD="VIDER"
        ;;
esac

echo "Confirmez le vidage en tapant '$CONFIRM_WORD' : "
read -r CONFIRMATION

if [ "$CONFIRMATION" != "$CONFIRM_WORD" ]; then
    echo "❌ Vidage annulé."
    exit 0
fi

echo ""
echo "Vidage en cours..."

# Effectuer le vidage selon la cible
case $TARGET in
    today)
        sqlite3 "$DB_PATH" <<EOF
BEGIN TRANSACTION;
DELETE FROM events_today;
DELETE FROM sqlite_sequence WHERE name='events_today';
COMMIT;
VACUUM;
EOF
        echo "✓ Table events_today vidée"
        ;;
    history)
        sqlite3 "$DB_PATH" <<EOF
BEGIN TRANSACTION;
DELETE FROM events_history;
DELETE FROM sqlite_sequence WHERE name='events_history';
COMMIT;
VACUUM;
EOF
        echo "✓ Table events_history vidée"
        ;;
    all)
        sqlite3 "$DB_PATH" <<EOF
BEGIN TRANSACTION;
DELETE FROM events_today;
DELETE FROM events_history;
DELETE FROM sqlite_sequence WHERE name IN ('events_today', 'events_history');
COMMIT;
VACUUM;
EOF
        echo "✓ Tables events_today et events_history vidées"
        ;;
esac

echo "✓ Auto-increment réinitialisé"
echo "✓ Base optimisée (VACUUM)"

# Nouvelle taille
NEW_FILE_SIZE=$(du -h "$DB_PATH" | cut -f1)

echo ""
echo "=== Vidage terminé ==="
echo "Nouvelle taille : $NEW_FILE_SIZE"
echo "Ancienne taille : $FILE_SIZE"
echo ""

# Vérifier le résultat
NEW_TODAY=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM events_today")
NEW_HISTORY=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM events_history")

echo "Vérification :"
echo "  - events_today : $NEW_TODAY enregistrements"
echo "  - events_history : $NEW_HISTORY enregistrements"
echo ""
echo "✓ La base est maintenant prête à recevoir de nouvelles données."
