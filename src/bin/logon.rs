//! # Binaire Logon
//!
//! Ce binaire est exécuté lors de l'ouverture d'une session Windows.
//! Il collecte les informations de base et les envoie au serveur de monitoring.

use winlog::{
    http_client::MonitoringClient,
    system_info,
    data_structures::WinlogData,
    utils,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("[LOGON] Démarrage du processus d'ouverture de session");
    
    // Collecte des informations système de base
    let system_info = system_info::get_basic_system_info();
    
    // Création de la structure de données
    let mut data = WinlogData::new(
        system_info.get("username").unwrap_or(&"unknown".to_string()).clone(),
        "login".to_string(),
    );
    
    // Ajout des informations système
    data.hostname = system_info.get("hostname").unwrap_or(&"unknown".to_string()).clone();
    data.os_info = system_info;
    
    // Validation des données
    if !utils::validate_data(&data) {
        eprintln!("[LOGON] Erreur: Données invalides");
        return Err("Données invalides".into());
    }
    
    // TODO: Envoi des données au serveur
    println!("[LOGON] Données à envoyer: {:?}", data);
    println!("[LOGON] Session ouverte avec succès");
    
    Ok(())
}