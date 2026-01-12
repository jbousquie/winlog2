//! # Binaire Logon
//!
//! Ce binaire est exécuté lors de l'ouverture d'une session Windows.
//! Il collecte les informations de base et les envoie au serveur de monitoring.

use winlog::utils;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Action "C" = Connexion
    utils::process_session_event("C", "logon")
}