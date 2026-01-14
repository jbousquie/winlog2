#!/bin/bash
# Script de vérification et compilation pour Linux
# Usage : ./build-linux.sh

set -e  # Arrêt en cas d'erreur

echo -e "\033[1;36m=== Winlog 2 - Script de compilation Linux ===\033[0m"
echo ""

# 1. Vérification Rust
echo -e "\033[1;33m[1/5] Vérification de Rust...\033[0m"
if command -v rustc &> /dev/null; then
    RUST_VERSION=$(rustc --version)
    echo -e "  \033[1;32m✓ Rust installé : $RUST_VERSION\033[0m"
else
    echo -e "  \033[1;31m✗ Rust non installé !\033[0m"
    echo -e "  \033[1;33mInstaller avec : curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh\033[0m"
    exit 1
fi

# 2. Vérification toolchain
echo -e "\033[1;33m[2/5] Vérification de la toolchain...\033[0m"
TOOLCHAIN=$(rustup show active-toolchain)
echo -e "  \033[1;32m✓ Toolchain active : $TOOLCHAIN\033[0m"

# 3. Vérification du linker
echo -e "\033[1;33m[3/5] Vérification du linker...\033[0m"
if command -v gcc &> /dev/null; then
    GCC_VERSION=$(gcc --version | head -n1)
    echo -e "  \033[1;32m✓ GCC détecté : $GCC_VERSION\033[0m"
else
    echo -e "  \033[1;31m✗ GCC non installé !\033[0m"
    echo -e "  \033[1;33mInstaller avec : sudo apt install build-essential\033[0m"
    exit 1
fi

# 4. Compilation du client
echo -e "\033[1;33m[4/5] Compilation du client...\033[0m"
pushd client > /dev/null
if cargo build --release > /dev/null 2>&1; then
    echo -e "  \033[1;32m✓ Client compilé avec succès\033[0m"
    echo -e "    \033[0;90m- target/release/logon\033[0m"
    echo -e "    \033[0;90m- target/release/logout\033[0m"
    echo -e "    \033[0;90m- target/release/matos\033[0m"
else
    echo -e "  \033[1;31m✗ Erreur de compilation du client\033[0m"
    popd > /dev/null
    exit 1
fi
popd > /dev/null

# 5. Compilation du serveur
echo -e "\033[1;33m[5/5] Compilation du serveur...\033[0m"
pushd serveur > /dev/null
if cargo build --release > /dev/null 2>&1; then
    echo -e "  \033[1;32m✓ Serveur compilé avec succès\033[0m"
    echo -e "    \033[0;90m- target/release/winlog-server\033[0m"
else
    echo -e "  \033[1;31m✗ Erreur de compilation du serveur\033[0m"
    popd > /dev/null
    exit 1
fi
popd > /dev/null

# Résumé final
echo ""
echo -e "\033[1;32m=== Compilation terminée avec succès ! ===\033[0m"
echo ""
echo -e "\033[1;36mBinaires disponibles :\033[0m"
echo -e "  \033[1;33mClient :\033[0m"
echo -e "    \033[1;37m- client/target/release/logon\033[0m"
echo -e "    \033[1;37m- client/target/release/logout\033[0m"
echo -e "    \033[1;37m- client/target/release/matos\033[0m"
echo -e "  \033[1;33mServeur :\033[0m"
echo -e "    \033[1;37m- serveur/target/release/winlog-server\033[0m"
echo ""
echo -e "\033[1;36mProchaines étapes :\033[0m"
echo -e "  \033[1;37m1. Configurer serveur/config.toml\033[0m"
echo -e "  \033[1;37m2. Créer la base SQLite : serveur/scripts/create_db.sh\033[0m"
echo -e "  \033[1;37m3. Démarrer le serveur : serveur/target/release/winlog-server\033[0m"
echo ""
