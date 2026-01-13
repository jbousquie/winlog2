//! # Binaire Logout
//!
//! Ce binaire est exécuté lors de la fermeture d'une session (Windows/Linux).
//! Il collecte les informations de base et les envoie au serveur de monitoring.

use winlog_client::utils;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Action "D" = Déconnexion
    utils::process_session_event("D", "logout")
}