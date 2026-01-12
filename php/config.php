<?php
/**
 * Configuration commune pour tous les scripts Winlog
 * Centralise les paramètres de base de données et autres constantes
 */

// Configuration de la base de données SQLite
const DB_PATH = '/var/www/ferron/winlog/data/winlog.db';
const DB_DIR = '/var/www/ferron/winlog/data';

// Configuration du serveur HTTP
const EXPECTED_USER_AGENT = 'Winlog/0.1.0 (Windows)';
const VALID_ACTIONS = ['C', 'D', 'M']; // Connexion, Déconnexion, Matériel

// Configuration SQLite optimale
const SQLITE_PRAGMA_CONFIG = [
    "PRAGMA journal_mode = WAL",
    "PRAGMA synchronous = NORMAL", 
    "PRAGMA busy_timeout = 30000",
    "PRAGMA cache_size = 10000"
];

?>