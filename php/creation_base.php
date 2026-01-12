<?php
/**
 * Script d'initialisation de la base SQLite Winlog
 * Crée la base de données et les tables nécessaires
 * 
 * Usage: php creation_base.php
 */

// Import de la configuration commune
require_once 'config.php';

echo "=== Création de la base Winlog ===\n";

// Vérification des extensions PHP requises
if (!extension_loaded('pdo')) {
    die("❌ Extension PDO non disponible\n");
}
if (!extension_loaded('pdo_sqlite')) {
    die("❌ Extension PDO SQLite non disponible. Installez: php-sqlite3 php-pdo-sqlite\n");
}
echo "✓ Extensions PDO et SQLite disponibles\n";

// Debug des permissions et utilisateur
echo "Utilisateur PHP : " . (function_exists('posix_getpwuid') ? posix_getpwuid(posix_geteuid())['name'] : 'inconnu') . "\n";
echo "Répertoire cible : " . DB_DIR . "\n";
echo "Permissions parent : " . substr(sprintf('%o', fileperms(dirname(DB_DIR))), -4) . "\n";

// Vérifier et créer le répertoire
if (!is_dir(DB_DIR)) {
    echo "Le répertoire n'existe pas, tentative de création...\n";
    if (!mkdir(DB_DIR, 0755, true)) {
        die("Erreur: Impossible de créer le répertoire " . DB_DIR . "\n");
    }
    echo "✓ Répertoire créé : " . DB_DIR . "\n";
} else {
    echo "✓ Répertoire existant : " . DB_DIR . "\n";
    echo "Permissions actuelles : " . substr(sprintf('%o', fileperms(DB_DIR)), -4) . "\n";
}

// Vérifier si la base existe déjà
if (file_exists(DB_PATH)) {
    echo "⚠ Base de données déjà existante : " . DB_PATH . "\n";
    echo "Utilisez purge_base.php pour vider ou delete_base.php pour supprimer\n";
    exit(1);
}

try {
    // Créer la connexion SQLite
    $pdo = new PDO('sqlite:' . DB_PATH);
    $pdo->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
    
    echo "✓ Connexion SQLite établie\n";
    
    // Configuration optimale SQLite
    foreach (SQLITE_PRAGMA_CONFIG as $pragma) {
        $pdo->exec($pragma);
    }
    
    echo "✓ Configuration SQLite optimisée\n";
    
    // Création de la table principale
    $sqlTable = "
    CREATE TABLE IF NOT EXISTS events (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        username VARCHAR(50) NOT NULL,
        action CHAR(1) NOT NULL CHECK (action IN ('C', 'D', 'M')),
        timestamp DATETIME NOT NULL,
        hostname VARCHAR(100),
        source_ip VARCHAR(45),
        server_timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
        
        -- Informations OS (extraites du JSON os_info)
        os_name VARCHAR(50),
        os_version VARCHAR(100),
        kernel_version VARCHAR(50),
        
        -- Informations matérielles (JSON complet pour action='M')
        hardware_info TEXT,
        
        -- Identifiant unique de session
        session_uuid VARCHAR(100),
        
        -- Metadata
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP
    )";
    
    $pdo->exec($sqlTable);
    echo "✓ Table 'events' créée\n";
    
    // Création des index pour performances
    $indexes = [
        "CREATE INDEX IF NOT EXISTS idx_username_action ON events(username, action)",
        "CREATE INDEX IF NOT EXISTS idx_timestamp ON events(timestamp)", 
        "CREATE INDEX IF NOT EXISTS idx_hostname ON events(hostname)",
        "CREATE INDEX IF NOT EXISTS idx_action_timestamp ON events(action, timestamp)",
        "CREATE INDEX IF NOT EXISTS idx_session_uuid ON events(session_uuid)",
        "CREATE INDEX IF NOT EXISTS idx_source_ip ON events(source_ip)"
    ];
    
    foreach ($indexes as $index) {
        $pdo->exec($index);
    }
    echo "✓ Index créés (" . count($indexes) . ")\n";
    
    // Informations finales
    $fileSize = filesize(DB_PATH);
    $permissions = substr(sprintf('%o', fileperms(DB_PATH)), -4);
    
    echo "\n=== Création terminée avec succès ===\n";
    echo "Base de données : " . DB_PATH . "\n";
    echo "Taille : " . $fileSize . " octets\n";
    echo "Permissions : " . $permissions . "\n";
    echo "\nLa base est prête à recevoir les données Winlog !\n";
    
} catch (Exception $e) {
    echo "❌ Erreur lors de la création : " . $e->getMessage() . "\n";
    exit(1);
}
?>