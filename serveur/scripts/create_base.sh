#!/bin/bash
###############################################################################
# Script de création de la base SQLite Winlog avec structure partitionnée
# Crée deux tables : events_today (données du jour) et events_history (archive)
#
# Usage: ./create_base.sh
###############################################################################

set -e  # Arrêt immédiat en cas d'erreur

# Configuration
# Chemin relatif au répertoire du projet
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
DB_DIR="$PROJECT_DIR/data"
DB_PATH="$DB_DIR/winlog.db"

echo "=== Création de la base Winlog (structure partitionnée) ==="
echo ""

# Vérifier que sqlite3 est installé
if ! command -v sqlite3 &> /dev/null; then
    echo "❌ Erreur : sqlite3 n'est pas installé"
    echo "   Installez-le avec : sudo apt install sqlite3"
    exit 1
fi
echo "✓ SQLite3 disponible ($(sqlite3 --version | cut -d' ' -f1-2))"

# Afficher l'utilisateur courant
echo "✓ Utilisateur : $(whoami)"

# Créer le répertoire si nécessaire
if [ ! -d "$DB_DIR" ]; then
    echo "Le répertoire n'existe pas, création..."
    mkdir -p "$DB_DIR" || {
        echo "❌ Erreur : Impossible de créer le répertoire $DB_DIR"
        echo "   Exécutez avec sudo si nécessaire"
        exit 1
    }
    chmod 755 "$DB_DIR"
    echo "✓ Répertoire créé : $DB_DIR"
else
    echo "✓ Répertoire existant : $DB_DIR"
    echo "  Permissions : $(stat -c '%a' "$DB_DIR")"
fi

# Vérifier si la base existe déjà
if [ -f "$DB_PATH" ]; then
    echo ""
    echo "⚠ Base de données déjà existante : $DB_PATH"
    echo "  Taille actuelle : $(du -h "$DB_PATH" | cut -f1)"
    echo ""
    echo "Utilisez :"
    echo "  - ./purge_base.sh pour vider les données"
    echo "  - ./delete_base.sh pour supprimer complètement"
    exit 1
fi

# Création de la base avec les deux tables
echo ""
echo "Création de la structure de la base..."

sqlite3 "$DB_PATH" <<'EOF'
-- Configuration optimale SQLite
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA busy_timeout = 30000;
PRAGMA cache_size = 10000;
PRAGMA foreign_keys = ON;

-- ============================================================================
-- Table : events_today (données du jour - haute performance)
-- ============================================================================
CREATE TABLE IF NOT EXISTS events_today (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username VARCHAR(50) NOT NULL,
    action CHAR(1) NOT NULL CHECK (action IN ('C', 'D', 'M')),
    timestamp DATETIME NOT NULL,
    hostname VARCHAR(100),
    source_ip VARCHAR(45),
    server_timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    
    -- Informations OS
    os_name VARCHAR(50),
    os_version VARCHAR(100),
    kernel_version VARCHAR(50),
    
    -- Informations matérielles (JSON pour action='M')
    hardware_info TEXT,
    
    -- Identifiant unique de session
    session_uuid VARCHAR(100),
    
    -- Metadata
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Index optimisés pour recherche rapide (sessions actives)
CREATE INDEX IF NOT EXISTS idx_today_username_action ON events_today(username, action);
CREATE INDEX IF NOT EXISTS idx_today_timestamp ON events_today(timestamp);
CREATE INDEX IF NOT EXISTS idx_today_hostname ON events_today(hostname);
CREATE INDEX IF NOT EXISTS idx_today_session ON events_today(session_uuid);
CREATE INDEX IF NOT EXISTS idx_today_action_time ON events_today(action, timestamp);

-- ============================================================================
-- Table : events_history (archive - toutes les données passées)
-- ============================================================================
CREATE TABLE IF NOT EXISTS events_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username VARCHAR(50) NOT NULL,
    action CHAR(1) NOT NULL CHECK (action IN ('C', 'D', 'M')),
    timestamp DATETIME NOT NULL,
    hostname VARCHAR(100),
    source_ip VARCHAR(45),
    server_timestamp DATETIME,
    
    -- Informations OS
    os_name VARCHAR(50),
    os_version VARCHAR(100),
    kernel_version VARCHAR(50),
    
    -- Informations matérielles
    hardware_info TEXT,
    
    -- Session
    session_uuid VARCHAR(100),
    
    -- Metadata
    created_at DATETIME,
    archived_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Index pour recherche dans l'historique
CREATE INDEX IF NOT EXISTS idx_history_username ON events_history(username);
CREATE INDEX IF NOT EXISTS idx_history_timestamp ON events_history(timestamp);
CREATE INDEX IF NOT EXISTS idx_history_hostname ON events_history(hostname);
CREATE INDEX IF NOT EXISTS idx_history_session ON events_history(session_uuid);
CREATE INDEX IF NOT EXISTS idx_history_date ON events_history(DATE(timestamp));

-- ============================================================================
-- Vue combinée (pour requêtes globales)
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

EOF

if [ $? -eq 0 ]; then
    echo "✓ Tables créées : events_today, events_history"
    echo "✓ Vue créée : events_all"
    echo "✓ Index créés (11 au total)"
    
    # Afficher les informations finales
    echo ""
    echo "=== Création terminée avec succès ==="
    echo "Base de données : $DB_PATH"
    echo "Taille : $(du -h "$DB_PATH" | cut -f1)"
    echo "Permissions : $(stat -c '%a' "$DB_PATH")"
    
    # Afficher la structure
    echo ""
    echo "Structure de la base :"
    sqlite3 "$DB_PATH" ".tables"
    
    echo ""
    echo "✓ La base est prête à recevoir les données Winlog !"
    echo ""
    echo "Notes :"
    echo "  - events_today : Données du jour (auto-nettoyée quotidiennement)"
    echo "  - events_history : Archive de toutes les données"
    echo "  - events_all : Vue combinée pour requêtes globales"
    echo ""
    echo "Prochaines étapes :"
    echo "  1. Configurer la rotation quotidienne : ./rotate_daily.sh"
    echo "  2. Adapter index.php pour utiliser events_today"
else
    echo "❌ Erreur lors de la création de la base"
    exit 1
fi
