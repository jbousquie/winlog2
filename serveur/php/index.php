<?php
/**
 * Serveur de réception pour les données Winlog
 * Traite les requêtes POST JSON des clients logon, logout et matos
 * Stockage dans base SQLite avec ajout de l'IP source
 */

// Import de la configuration commune et des requêtes SQL
require_once 'config.php';
require_once 'index_sql.php';

// Headers de sécurité
header('Content-Type: application/json');
header('X-Content-Type-Options: nosniff');
header('X-Frame-Options: DENY');


/**
 * Fonction pour générer un identifiant de session simple
 */
function generateSessionId($username, $hostname, $timestamp) {
    // Format simple: username@hostname@date_hash
    $date = date('Y-m-d', strtotime($timestamp));
    $hash = substr(hash('md5', $username . $hostname . $date . microtime()), 0, 6);
    return $username . '@' . $hostname . '@' . $hash;
}

/**
 * Fonction pour trouver une session ouverte le même jour
 */
function findOpenSessionToday($pdo, $username, $hostname, $timestamp) {
    $stmt = $pdo->prepare(SQL_FIND_OPEN_SESSION_TODAY);
    $stmt->execute([$username, $hostname, $timestamp]);
    return $stmt->fetch(PDO::FETCH_ASSOC);
}

/**
 * Fonction pour trouver la dernière session ouverte (pour les déconnexions)
 */
function findLastOpenSession($pdo, $username, $hostname) {
    $stmt = $pdo->prepare(SQL_FIND_LAST_OPEN_SESSION);
    $stmt->execute([$username, $hostname]);
    return $stmt->fetchColumn();
}

/**
 * Fonction pour se connecter à la base SQLite
 */
function getDbConnection() {
    try {
        $pdo = new PDO('sqlite:' . DB_PATH);
        $pdo->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
        
        // Configuration optimale
        foreach (SQLITE_PRAGMA_CONFIG as $pragma) {
            $pdo->exec($pragma);
        }
        
        return $pdo;
    } catch (Exception $e) {
        logError("Erreur connexion SQLite: " . $e->getMessage());
        throw $e;
    }
}

/**
 * Fonction pour logger les erreurs
 */
function logError($message) {
    error_log("[Winlog] " . date('Y-m-d H:i:s') . " - " . $message);
}

/**
 * Fonction pour valider la structure JSON reçue
 */
function validateJsonStructure($data) {
    // Vérification des champs obligatoires
    $requiredFields = ['username', 'action', 'timestamp'];
    
    foreach ($requiredFields as $field) {
        if (!isset($data[$field]) || empty($data[$field])) {
            return false;
        }
    }
    
    // Validation du code d'action
    if (!in_array($data['action'], VALID_ACTIONS)) {
        return false;
    }
    
    // Validation du timestamp (format ISO 8601 - plus souple)
    // Accepte les formats avec T et Z (RFC 3339)
    if (!preg_match('/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}/', $data['timestamp'])) {
        return false;
    }
    
    return true;
}

/**
 * Fonction pour obtenir l'adresse IP réelle du client
 */
function getRealIpAddress() {
    $headers = [
        'HTTP_CF_CONNECTING_IP',     // Cloudflare
        'HTTP_CLIENT_IP',            // Proxy
        'HTTP_X_FORWARDED_FOR',      // Load balancer/proxy
        'HTTP_X_FORWARDED',          // Proxy
        'HTTP_X_CLUSTER_CLIENT_IP',  // Cluster
        'HTTP_FORWARDED_FOR',        // Proxy
        'HTTP_FORWARDED',            // Proxy
        'REMOTE_ADDR'                // Standard
    ];
    
    foreach ($headers as $header) {
        if (!empty($_SERVER[$header])) {
            $ips = explode(',', $_SERVER[$header]);
            $ip = trim($ips[0]);
            
            // Validation de l'adresse IP
            if (filter_var($ip, FILTER_VALIDATE_IP, FILTER_FLAG_NO_PRIV_RANGE | FILTER_FLAG_NO_RES_RANGE)) {
                return $ip;
            }
        }
    }
    
    // Fallback vers REMOTE_ADDR même si privé
    return $_SERVER['REMOTE_ADDR'] ?? 'unknown';
}

// Vérification de la méthode HTTP
if ($_SERVER['REQUEST_METHOD'] !== 'POST') {
    http_response_code(405);
    echo json_encode(['error' => 'Method not allowed', 'expected' => 'POST']);
    exit;
}

// Vérification du User-Agent
$userAgent = $_SERVER['HTTP_USER_AGENT'] ?? '';
if ($userAgent !== EXPECTED_USER_AGENT) {
    logError("Invalid User-Agent: " . $userAgent);
    http_response_code(403);
    echo json_encode(['error' => 'Invalid User-Agent']);
    exit;
}

// Vérification du Content-Type
$contentType = $_SERVER['CONTENT_TYPE'] ?? '';
if (strpos($contentType, 'application/json') !== 0) {
    logError("Invalid Content-Type: " . $contentType);
    http_response_code(400);
    echo json_encode(['error' => 'Invalid Content-Type', 'expected' => 'application/json']);
    exit;
}

// Lecture des données JSON
$jsonInput = file_get_contents('php://input');
if (empty($jsonInput)) {
    logError("Empty request body");
    http_response_code(400);
    echo json_encode(['error' => 'Empty request body']);
    exit;
}

// Décodage JSON
$data = json_decode($jsonInput, true);
if (json_last_error() !== JSON_ERROR_NONE) {
    logError("JSON decode error: " . json_last_error_msg());
    logError("Received data: " . substr($jsonInput, 0, 500)); // Log premiers 500 caractères
    http_response_code(400);
    echo json_encode(['error' => 'Invalid JSON', 'details' => json_last_error_msg()]);
    exit;
}

// Debug: Log de la structure reçue (uniquement les champs clés)
$debugData = [
    'username' => $data['username'] ?? 'missing',
    'action' => $data['action'] ?? 'missing', 
    'timestamp' => $data['timestamp'] ?? 'missing',
    'hostname' => $data['hostname'] ?? 'not provided'
];
logError("Received JSON structure: " . json_encode($debugData));

// Validation de la structure
if (!validateJsonStructure($data)) {
    $errorDetails = [
        'username' => isset($data['username']) ? 'OK' : 'MISSING',
        'action' => isset($data['action']) ? ($data['action'] ?? 'EMPTY') : 'MISSING',
        'timestamp' => isset($data['timestamp']) ? 'OK' : 'MISSING',
        'action_valid' => isset($data['action']) ? (in_array($data['action'], VALID_ACTIONS) ? 'YES' : 'NO') : 'N/A'
    ];
    logError("Invalid JSON structure: " . json_encode($errorDetails));
    logError("Full received data: " . json_encode($data));
    http_response_code(400);
    echo json_encode(['error' => 'Invalid JSON structure', 'details' => $errorDetails]);
    exit;
}

// Ajout de l'adresse IP source
$data['source_ip'] = getRealIpAddress();
$data['server_timestamp'] = date('c'); // ISO 8601

// Traitement de la session
try {
    $pdo = getDbConnection();
    $pdo->beginTransaction();
    
    $session_uuid = null;
    
    if ($data['action'] === 'C') {
        // Nouvelle connexion - vérifier s'il y a une session ouverte aujourd'hui
        $openSession = findOpenSessionToday($pdo, $data['username'], $data['hostname'], $data['timestamp']);
        
        if ($openSession) {
            // Il y a une session ouverte - la fermer automatiquement
            logError("Session ouverte détectée pour " . $data['username'] . "@" . $data['hostname'] . " - fermeture automatique");
            
            // Créer un timestamp de déconnexion automatique (1 seconde avant la nouvelle connexion)
            $autoDisconnectTime = date('c', strtotime($data['timestamp']) - 1);
            
            // Insérer la déconnexion automatique
            $stmt = $pdo->prepare(SQL_INSERT_AUTO_DISCONNECT);
            $stmt->execute([
                $data['username'],
                $autoDisconnectTime,
                $data['hostname'],
                $data['source_ip'],
                $data['server_timestamp'],
                $data['os_info']['os_name'] ?? null,
                $data['os_info']['os_version'] ?? null,
                $data['os_info']['kernel_version'] ?? null,
                $openSession['session_uuid']
            ]);
            
            logError("Déconnexion automatique insérée pour session: " . $openSession['session_uuid']);
        }
        
        // Générer un nouvel UUID pour la nouvelle connexion
        $session_uuid = generateSessionId($data['username'], $data['hostname'], $data['timestamp']);
        
    } elseif ($data['action'] === 'D') {
        // Déconnexion - trouver la dernière session ouverte
        $session_uuid = findLastOpenSession($pdo, $data['username'], $data['hostname']);
        if (!$session_uuid) {
            logError("Aucune session ouverte trouvée pour " . $data['username'] . "@" . $data['hostname']);
            // Créer un UUID temporaire pour éviter les erreurs
            $session_uuid = 'orphan_' . generateSessionId($data['username'], $data['hostname'], $data['timestamp']);
        }
    } else { // Action 'M' (matériel)
        $session_uuid = 'hardware_' . generateSessionId($data['username'], $data['hostname'], $data['timestamp']);
    }
    
    // Extraction des informations OS
    $os_name = $data['os_info']['os_name'] ?? null;
    $os_version = $data['os_info']['os_version'] ?? null; 
    $kernel_version = $data['os_info']['kernel_version'] ?? null;
    
    // Préparation des données matérielles (JSON)
    $hardware_info = null;
    if (isset($data['hardware_info']) && $data['hardware_info'] !== null) {
        $hardware_info = json_encode($data['hardware_info']);
    }
    
    // Insertion en base
    $stmt = $pdo->prepare(SQL_INSERT_EVENT);
    
    $result = $stmt->execute([
        $data['username'],
        $data['action'],
        $data['timestamp'],
        $data['hostname'],
        $data['source_ip'],
        $data['server_timestamp'],
        $os_name,
        $os_version,
        $kernel_version,
        $hardware_info,
        $session_uuid
    ]);
    
    if (!$result) {
        throw new Exception("Erreur lors de l'insertion en base");
    }
    
    $pdo->commit();
    $eventId = $pdo->lastInsertId();
    
} catch (Exception $e) {
    if (isset($pdo)) {
        $pdo->rollback();
    }
    logError("Erreur SQLite: " . $e->getMessage());
    http_response_code(500);
    echo json_encode(['error' => 'Database error']);
    exit;
}

// Réponse de succès
http_response_code(200);
echo json_encode([
    'status' => 'success',
    'message' => 'Data stored in database',
    'event_id' => $eventId,
    'session_uuid' => $session_uuid,
    'action' => $data['action'],
    'username' => $data['username']
]);

// Log de succès
error_log("[Winlog] Data stored: ID=" . $eventId . " - " . $data['username'] . " - " . $data['action'] . " - Session: " . $session_uuid . " from " . $data['source_ip']);
?>