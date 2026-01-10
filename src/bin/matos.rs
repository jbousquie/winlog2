//! # Binaire Matos
//!
//! Ce binaire collecte des informations détaillées sur le matériel du système.
//! Il peut être exécuté périodiquement ou sur demande.

use winlog::{
    http_client::MonitoringClient,
    system_info,
    data_structures::WinlogData,
    utils,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("[MATOS] Démarrage de la collecte d'informations matérielles");
    
    // Collecte des informations système de base
    let basic_info = system_info::get_basic_system_info();
    
    // Collecte des informations matérielles détaillées
    let hardware_info = system_info::get_hardware_info();
    
    // Création de la structure de données
    let mut data = WinlogData::new(
        basic_info.get("username").unwrap_or(&"system".to_string()).clone(),
        "hardware_info".to_string(),
    );
    
    // Ajout des informations système et matérielles
    data.hostname = basic_info.get("hostname").unwrap_or(&"unknown".to_string()).clone();
    data.os_info = basic_info;
    data.hardware_info = Some(hardware_info);
    
    // Validation des données
    if !utils::validate_data(&data) {
        eprintln!("[MATOS] Erreur: Données invalides");
        return Err("Données invalides".into());
    }
    
    // TODO: Envoi des données au serveur
    println!("[MATOS] Données matérielles à envoyer: {:?}", data);
    println!("[MATOS] Collecte matérielle terminée avec succès");
    
    Ok(())
}