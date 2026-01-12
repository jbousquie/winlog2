<?php
/**
 * Serveur de réception pour les données Winlog
 * Traite les requêtes POST JSON des clients logon, logout et matos
 * Stockage dans winlog.log avec ajout de l'IP source
 */

// Configuration
const EXPECTED_USER_AGENT = 'Winlog/0.1.0 (Windows)';
const LOG_FILE = 'winlog.log';
const VALID_ACTIONS = ['C', 'D', 'M']; // Connexion, Déconnexion, Matériel

// Headers de sécurité
header('Content-Type: application/json');
header('X-Content-Type-Options: nosniff');
header('X-Frame-Options: DENY');


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

// Préparation de la ligne de log
$logLine = json_encode($data, JSON_UNESCAPED_SLASHES | JSON_UNESCAPED_UNICODE) . "\n";

// Écriture dans le fichier de log
$result = file_put_contents(LOG_FILE, $logLine, FILE_APPEND | LOCK_EX);
if ($result === false) {
    logError("Failed to write to log file");
    http_response_code(500);
    echo json_encode(['error' => 'Internal server error']);
    exit;
}

// Réponse de succès
http_response_code(200);
echo json_encode([
    'status' => 'success',
    'message' => 'Data logged successfully',
    'action' => $data['action'],
    'username' => $data['username']
]);

// Log de succès
error_log("[Winlog] Data logged: " . $data['username'] . " - " . $data['action'] . " from " . $data['source_ip']);
?>