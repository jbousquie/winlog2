//! # Winlog 2 - Librairie partagée
//! 
//! Cette librairie contient la logique commune utilisée par les 3 binaires
//! (logon, logout, matos) pour collecter et envoyer les informations système.

// Déclaration du module de configuration externe
pub mod config;

/// Module de gestion des requêtes HTTP vers le serveur de monitoring
pub mod http_client {
    use reqwest::Client;
    use std::collections::HashMap;
    use crate::config;
    
    /// Client HTTP pour envoyer les données au serveur de monitoring
    pub struct MonitoringClient {
        client: Client,
        server_url: String,
    }
    
    impl MonitoringClient {
        /// Crée une nouvelle instance du client HTTP
        pub fn new(server_url: Option<String>) -> Self {
            Self {
                client: Client::new(),
                server_url: server_url.unwrap_or_else(|| config::DEFAULT_SERVER_URL.to_string()),
            }
        }
        
        /// Envoie les données au serveur via HTTP POST
        pub async fn send_data(&self, data: HashMap<String, serde_json::Value>) -> Result<(), Box<dyn std::error::Error>> {
            // TODO: Implémentation de l'envoi HTTP POST avec retry selon config::MAX_RETRIES
            println!("Envoi des données vers {}: {:?}", self.server_url, data);
            println!("Configuration: timeout={}s, max_retries={}", 
                    config::DEFAULT_TIMEOUT, config::MAX_RETRIES);
            Ok(())
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
        let sys = System::new_all();
        
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
        pub fn new(username: String, action: String) -> Self {
            Self {
                username,
                action,
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
    /// Génère un timestamp au format ISO 8601 UTC
    pub fn get_current_timestamp() -> String {
        chrono::Utc::now().to_rfc3339()
    }
    
    /// Valide les données avant envoi
    pub fn validate_data(data: &crate::data_structures::WinlogData) -> bool {
        !data.username.is_empty() && !data.action.is_empty()
    }
}