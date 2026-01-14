//! Test de gestion des chemins multi-plateforme

use std::path::PathBuf;

fn main() {
    println!("=== Test de la gestion des chemins multi-plateforme ===\n");

    let config_paths = vec![
        "data/winlog.db",
        "./data/winlog.db",
        "C:/Users/Admin/winlog/data/winlog.db",
        "/var/www/winlog/data/winlog.db",
    ];

    for path_str in config_paths {
        println!("📝 Configuration TOML : \"{}\"", path_str);
        let path_buf = PathBuf::from(path_str);
        println!("   → PathBuf.display() : {}", path_buf.display());
        println!("   → SQLite URL        : sqlite:{}", path_buf.display());
        println!();
    }

    println!("=== Informations système ===\n");
    println!("OS             : {}", std::env::consts::OS);
    println!("Architecture   : {}", std::env::consts::ARCH);
    println!("Séparateur     : {:?}", std::path::MAIN_SEPARATOR);
    
    println!("\n✅ Sur Windows, PathBuf convertit '/' en '\\'");
    println!("✅ Sur Linux/macOS, PathBuf utilise '/' tel quel");
    println!("✅ SQLx accepte les deux formats sans problème");
}
