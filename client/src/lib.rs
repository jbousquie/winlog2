//! # Winlog 2 - Librairie partagée
//! 
//! Cette librairie contient la logique commune utilisée par les 3 binaires
//! (logon, logout, matos) pour collecter et envoyer les informations système.

// Déclaration du module de configuration externe
pub mod config;

/// Module de gestion des requêtes HTTP vers le serveur de monitoring
pub mod http_client {
    use minreq;
    use crate::config;
    
    /// Client HTTP pour envoyer les données au serveur de monitoring
    pub struct WinlogClient {
        server_url: String,
        #[allow(dead_code)]  // Timeout conservé pour évolutions futures
        timeout: std::time::Duration,
    }
    
    impl WinlogClient {
        /// Crée une nouvelle instance du client HTTP
        pub fn new(server_url: Option<String>) -> Self {
            Self {
                server_url: server_url.unwrap_or_else(|| config::DEFAULT_SERVER_URL.to_string()),
                timeout: std::time::Duration::from_secs(config::DEFAULT_TIMEOUT),
            }
        }
        
        /// Envoie les données au serveur via HTTP POST synchrone avec retry
        pub fn send_data(&self, data: &crate::data_structures::WinlogData) -> Result<(), Box<dyn std::error::Error>> {
            let json_data = serde_json::to_string(data)?;
            
            // Debug: Affichage du JSON envoyé
            println!("JSON envoyé: {}", json_data);
            
            for attempt in 1..=config::MAX_RETRIES {
                println!("Tentative {}/{} d'envoi vers {}", attempt, config::MAX_RETRIES, self.server_url);
                
                match minreq::post(&self.server_url)
                    .with_header("Content-Type", "application/json")
                    .with_header("User-Agent", config::USER_AGENT)
                    .with_timeout(config::DEFAULT_TIMEOUT)
                    .with_body(json_data.clone())
                    .send()
                {
                    Ok(response) => {
                        if response.status_code >= 200 && response.status_code < 300 {
                            println!("Données envoyées avec succès (HTTP {})", response.status_code);
                            return Ok(());
                        } else {
                            eprintln!("Erreur HTTP {}: {}", response.status_code, response.reason_phrase);
                            if let Ok(body) = response.as_str() {
                                eprintln!("Réponse du serveur: {}", body);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Erreur réseau (tentative {}): {}", attempt, e);
                    }
                }
                
                if attempt < config::MAX_RETRIES {
                    std::thread::sleep(std::time::Duration::from_millis(config::RETRY_DELAY_MS));
                }
            }
            
            Err("Échec d'envoi après tous les essais".into())
        }
    }
}

/// Module de collecte des informations système
pub mod system_info {
    use sysinfo::System;
    use std::collections::HashMap;
    
    /// Collecte les informations de base du système
    pub fn get_basic_system_info() -> HashMap<String, String> {
        let mut info = HashMap::new();
        
        // Username de l'utilisateur actuel
        info.insert("username".to_string(), whoami::username());
        
        // Hostname de la machine
        info.insert("hostname".to_string(), System::host_name().unwrap_or_default());
        
        // Informations OS
        info.insert("os_name".to_string(), System::name().unwrap_or_default());
        info.insert("os_version".to_string(), System::os_version().unwrap_or_default());
        info.insert("kernel_version".to_string(), System::kernel_version().unwrap_or_default());
        
        info
    }
    
    /// Collecte les informations matérielles détaillées
    pub fn get_hardware_info() -> HashMap<String, serde_json::Value> {
        let mut info = HashMap::new();
        let sys = System::new_all();
        
        // Informations CPU
        info.insert("cpu_count".to_string(), serde_json::Value::Number(sys.cpus().len().into()));
        if let Some(cpu) = sys.cpus().first() {
            info.insert("cpu_brand".to_string(), serde_json::Value::String(cpu.brand().to_string()));
            info.insert("cpu_frequency".to_string(), serde_json::Value::Number(cpu.frequency().into()));
        }
        
        // Informations mémoire
        info.insert("memory_total".to_string(), serde_json::Value::Number(sys.total_memory().into()));
        info.insert("memory_used".to_string(), serde_json::Value::Number(sys.used_memory().into()));
        
        info
    }
}

/// Module des structures de données
pub mod data_structures {
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;
    
    /// Structure principale des données à envoyer au serveur
    #[derive(Serialize, Deserialize, Debug)]
    pub struct WinlogData {
        pub username: String,
        pub action: String,
        pub timestamp: String,
        pub hostname: String,
        pub os_info: HashMap<String, String>,
        pub hardware_info: Option<HashMap<String, serde_json::Value>>,
    }
    
    impl WinlogData {
        /// Crée une nouvelle instance avec les informations de base
        pub fn new(username: String, action_code: String) -> Self {
            Self {
                username,
                action: action_code, // "C" = Connexion, "D" = Déconnexion, "M" = Matériel
                timestamp: chrono::Utc::now().to_rfc3339(),
                hostname: String::new(),
                os_info: HashMap::new(),
                hardware_info: None,
            }
        }
    }
}

/// Module des utilitaires communs
pub mod utils {
    use crate::{data_structures::WinlogData, http_client::WinlogClient, system_info};
    
    /// Génère un timestamp au format ISO 8601 UTC
    pub fn get_current_timestamp() -> String {
        chrono::Utc::now().to_rfc3339()
    }
    
    /// Valide les données avant envoi
    pub fn validate_data(data: &WinlogData) -> bool {
        !data.username.is_empty() && !data.action.is_empty()
    }
    
    /// Fonction commune pour traiter les événements de session (connexion/déconnexion)
    pub fn process_session_event(action_code: &str, event_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("[{}] Démarrage du processus {}", event_name.to_uppercase(), event_name);
        
        // Collecte des informations système de base
        let system_info = system_info::get_basic_system_info();
        
        // Extraction username - évite allocation temporaire avec map_or
        let username = system_info.get("username")
            .map(|s| s.as_str())
            .unwrap_or("unknown")
            .to_string();
        
        // Création de la structure de données
        let mut data = WinlogData::new(username, action_code.to_string());
        
        // Extraction hostname - évite allocation temporaire
        data.hostname = system_info.get("hostname")
            .map(|s| s.as_str())
            .unwrap_or("unknown")
            .to_string();
        
        data.os_info = system_info;
        
        // Validation des données
        if !validate_data(&data) {
            eprintln!("[{}] Erreur: Données invalides", event_name.to_uppercase());
            return Err("Données invalides".into());
        }
        
        // Envoi des données au serveur
        let client = WinlogClient::new(None);
        match client.send_data(&data) {
            Ok(()) => {
                let success_msg = match action_code {
                    "C" => "Session ouverte avec succès",
                    "D" => "Session fermée avec succès",
                    _ => "Opération terminée avec succès"
                };
                println!("[{}] {}", event_name.to_uppercase(), success_msg);
            }
            Err(e) => {
                eprintln!("[{}] Échec de l'envoi: {}", event_name.to_uppercase(), e);
                return Err(e);
            }
        }
        
        Ok(())
    }
    
    /// Fonction spécialisée pour traiter les informations matérielles
    pub fn process_hardware_info() -> Result<(), Box<dyn std::error::Error>> {
        println!("[MATOS] Démarrage de la collecte d'informations matérielles");
        
        // Collecte des informations système de base
        let basic_info = system_info::get_basic_system_info();
        
        // Collecte des informations matérielles détaillées
        let hardware_info = system_info::get_hardware_info();
        
        // Extraction username - évite allocation temporaire
        let username = basic_info.get("username")
            .map(|s| s.as_str())
            .unwrap_or("system")
            .to_string();
        
        // Création de la structure de données
        let mut data = WinlogData::new(username, "M".to_string());
        
        // Extraction hostname - évite allocation temporaire
        data.hostname = basic_info.get("hostname")
            .map(|s| s.as_str())
            .unwrap_or("unknown")
            .to_string();
        
        data.os_info = basic_info;
        data.hardware_info = Some(hardware_info);
        
        // Validation des données
        if !validate_data(&data) {
            eprintln!("[MATOS] Erreur: Données invalides");
            return Err("Données invalides".into());
        }
        
        // Envoi des données au serveur
        let client = WinlogClient::new(None);
        match client.send_data(&data) {
            Ok(()) => println!("[MATOS] Collecte matérielle terminée avec succès"),
            Err(e) => {
                eprintln!("[MATOS] Échec de l'envoi: {}", e);
                return Err(e);
            }
        }
        
        Ok(())
    }
}