<?php
/**
 * Requêtes SQL pour index.php
 * Centralisation de toutes les requêtes pour une meilleure lisibilité et maintenance
 */

// Requête pour trouver la dernière session ouverte (sans déconnexion correspondante)
const SQL_FIND_LAST_OPEN_SESSION = "
    SELECT session_uuid FROM events 
    WHERE username = ? AND hostname = ? AND action = 'C'
    AND NOT EXISTS (
        SELECT 1 FROM events e2 
        WHERE e2.session_uuid = events.session_uuid 
        AND e2.action = 'D'
    )
    ORDER BY timestamp DESC 
    LIMIT 1
";

// Requête pour chercher une connexion ouverte le même jour
const SQL_FIND_OPEN_SESSION_TODAY = "
    SELECT session_uuid, timestamp FROM events 
    WHERE username = ? AND hostname = ? AND action = 'C'
    AND DATE(timestamp) = DATE(?)
    AND NOT EXISTS (
        SELECT 1 FROM events e2 
        WHERE e2.session_uuid = events.session_uuid 
        AND e2.action = 'D'
    )
    ORDER BY timestamp DESC 
    LIMIT 1
";

// Requête d'insertion d'un événement de déconnexion automatique
const SQL_INSERT_AUTO_DISCONNECT = "
    INSERT INTO events (
        username, action, timestamp, hostname, source_ip, server_timestamp,
        os_name, os_version, kernel_version, session_uuid
    ) VALUES (?, 'D', ?, ?, ?, ?, ?, ?, ?, ?)
";

// Requête d'insertion d'un nouvel événement
const SQL_INSERT_EVENT = "
    INSERT INTO events (
        username, action, timestamp, hostname, source_ip, server_timestamp,
        os_name, os_version, kernel_version, hardware_info, session_uuid
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
";

// Requêtes utilitaires pour debug/administration (optionnelles)
const SQL_COUNT_EVENTS = "SELECT COUNT(*) as total FROM events";

const SQL_COUNT_EVENTS_BY_ACTION = "
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
";

const SQL_FIND_OPEN_SESSIONS = "
    SELECT username, hostname, session_uuid, timestamp, source_ip
    FROM events 
    WHERE action='C' 
    AND NOT EXISTS (
        SELECT 1 FROM events e2 
        WHERE e2.session_uuid = events.session_uuid 
        AND e2.action = 'D'
    )
    ORDER BY timestamp DESC
";

const SQL_SESSION_DURATION = "
    SELECT 
        c.username, c.hostname, c.session_uuid,
        c.timestamp as connexion,
        d.timestamp as deconnexion,
        (julianday(d.timestamp) - julianday(c.timestamp)) * 24 * 60 as duree_minutes
    FROM events c
    JOIN events d ON c.session_uuid = d.session_uuid
    WHERE c.action='C' AND d.action='D'
    ORDER BY c.timestamp DESC
";

?>