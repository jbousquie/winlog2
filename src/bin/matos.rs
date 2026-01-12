//! # Binaire Matos
//!
//! Ce binaire collecte des informations détaillées sur le matériel du système.
//! Il peut être exécuté périodiquement ou sur demande.

use winlog::utils;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Action "M" = Matériel
    utils::process_hardware_info()
}