<?php
/**
 * Script de vidage de la base SQLite Winlog
 * Supprime toutes les données mais conserve la structure
 * 
 * Usage: php purge_base.php
 * ATTENTION: Cette opération supprime toutes les données !
 */

// Import de la configuration commune
require_once 'config.php';

echo "=== Vidage de la base Winlog ===\n";
echo "⚠ ATTENTION: Cette opération va supprimer TOUTES les données !\n";
echo "La structure de la base sera conservée.\n";
echo "Base à vider : " . DB_PATH . "\n";

// Vérifier si la base existe
if (!file_exists(DB_PATH)) {
    echo "❌ Base de données introuvable : " . DB_PATH . "\n";
    echo "Utilisez creation_base.php pour créer la base.\n";
    exit(1);
}

try {
    // Connexion à la base
    $pdo = new PDO('sqlite:' . DB_PATH);
    $pdo->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
    
    echo "✓ Connexion établie\n";
    
    // Informations avant vidage
    $stmt = $pdo->query("SELECT COUNT(*) as total FROM events");
    $count = $stmt->fetch()['total'];
    
    if ($count == 0) {
        echo "ℹ La base est déjà vide (0 enregistrement).\n";
        exit(0);
    }
    
    echo "Enregistrements actuels : " . number_format($count) . "\n";
    
    // Statistiques par type d'action
    $stmt = $pdo->query("
        SELECT action, 
               CASE action 
                   WHEN 'C' THEN 'Connexions'
                   WHEN 'D' THEN 'Déconnexions' 
                   WHEN 'M' THEN 'Matériel'
                   ELSE 'Autre'
               END as type,
               COUNT(*) as nb
        FROM events 
        GROUP BY action
        ORDER BY action
    ");
    
    echo "\nRépartition par type :\n";
    while ($row = $stmt->fetch()) {
        echo "  " . $row['type'] . " (" . $row['action'] . ") : " . number_format($row['nb']) . "\n";
    }
    
    // Taille du fichier
    $fileSize = filesize(DB_PATH);
    echo "\nTaille actuelle : " . number_format($fileSize) . " octets\n";
    
    echo "\nConfirmation requise. Tapez 'VIDER' pour continuer : ";
    $confirmation = trim(fgets(STDIN));
    
    if ($confirmation !== 'VIDER') {
        echo "❌ Vidage annulé.\n";
        exit(0);
    }
    
    // Début de la transaction
    $pdo->beginTransaction();
    
    try {
        // Suppression de toutes les données
        $stmt = $pdo->prepare("DELETE FROM events");
        $deleted = $stmt->execute();
        
        // Réinitialiser l'auto-increment
        $pdo->exec("DELETE FROM sqlite_sequence WHERE name='events'");
        
        // Optimiser la base (récupérer l'espace)
        $pdo->exec("VACUUM");
        
        // Valider la transaction
        $pdo->commit();
        
        echo "✓ Toutes les données ont été supprimées\n";
        echo "✓ Auto-increment réinitialisé\n";
        echo "✓ Base optimisée (VACUUM)\n";
        
        // Nouvelle taille
        $newFileSize = filesize(DB_PATH);
        $savedBytes = $fileSize - $newFileSize;
        
        echo "\n=== Vidage terminé ===\n";
        echo "Nouvelle taille : " . number_format($newFileSize) . " octets\n";
        echo "Espace récupéré : " . number_format($savedBytes) . " octets\n";
        echo "La base est maintenant vide et prête à recevoir de nouvelles données.\n";
        
    } catch (Exception $e) {
        if (isset($pdo)) {
            $pdo->rollback();
        }
        throw $e;
    }
    
} catch (Exception $e) {
    echo "❌ Erreur lors du vidage : " . $e->getMessage() . "\n";
    exit(1);
}
?>