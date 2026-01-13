<?php
/**
 * Script de suppression de la base SQLite Winlog
 * Supprime complètement le fichier de base de données
 * 
 * Usage: php delete_base.php
 * ATTENTION: Cette opération est irréversible !
 */

// Import de la configuration commune
require_once 'config.php';

echo "=== Suppression de la base Winlog ===\n";
echo "⚠ ATTENTION: Cette opération va supprimer DÉFINITIVEMENT la base !\n";
echo "Base à supprimer : " . DB_PATH . "\n";

// Vérifier si la base existe
if (!file_exists(DB_PATH)) {
    echo "ℹ Base de données inexistante : " . DB_PATH . "\n";
    echo "Rien à supprimer.\n";
    exit(0);
}

// Afficher les informations avant suppression
$fileSize = filesize(DB_PATH);
echo "Taille actuelle : " . number_format($fileSize) . " octets\n";

try {
    // Tentative de connexion pour vérifier l'intégrité
    $pdo = new PDO('sqlite:' . DB_PATH, null, null, [PDO::ATTR_ERRMODE => PDO::ERRMODE_EXCEPTION]);
    
    // Compter les enregistrements
    $stmt = $pdo->query("SELECT COUNT(*) as total FROM events");
    $count = $stmt->fetch()['total'];
    echo "Enregistrements dans la base : " . number_format($count) . "\n";
    
    // Fermer la connexion avant suppression
    $pdo = null;
    
} catch (Exception $e) {
    echo "⚠ Impossible de lire la base (fichier corrompu?) : " . $e->getMessage() . "\n";
    echo "Suppression forcée...\n";
}

echo "\nConfirmation requise. Tapez 'SUPPRIMER' pour continuer : ";
$confirmation = trim(fgets(STDIN));

if ($confirmation !== 'SUPPRIMER') {
    echo "❌ Suppression annulée.\n";
    exit(0);
}

// Suppression du fichier
if (unlink(DB_PATH)) {
    echo "✓ Base de données supprimée avec succès !\n";
    
    // Supprimer les fichiers WAL et SHM si présents
    $walFile = DB_PATH . '-wal';
    $shmFile = DB_PATH . '-shm';
    
    if (file_exists($walFile)) {
        unlink($walFile);
        echo "✓ Fichier WAL supprimé\n";
    }
    
    if (file_exists($shmFile)) {
        unlink($shmFile);
        echo "✓ Fichier SHM supprimé\n";
    }
    
    echo "\n=== Suppression terminée ===\n";
    echo "Utilisez creation_base.php pour recréer la base.\n";
    
} else {
    echo "❌ Erreur lors de la suppression du fichier.\n";
    echo "Vérifiez les permissions du répertoire.\n";
    exit(1);
}
?>