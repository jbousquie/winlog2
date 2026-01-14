#!/usr/bin/env bash
# Script de vérification cross-platform pour Winlog 2
# Usage: ./check-build.sh [windows|linux|both]

set -e

TARGET=${1:-both}
COLOR_GREEN='\033[0;32m'
COLOR_BLUE='\033[0;34m'
COLOR_RED='\033[0;31m'
COLOR_RESET='\033[0m'

echo -e "${COLOR_BLUE}🔍 Vérification configuration cross-platform Winlog 2${COLOR_RESET}"
echo

# Vérifier rustup et toolchain
echo -e "${COLOR_BLUE}🦀 Configuration Rust:${COLOR_RESET}"
rustup show
echo

# Fonction de build et test
build_target() {
    local target=$1
    local name=$2
    
    echo -e "${COLOR_BLUE}🔨 Build $name ($target)...${COLOR_RESET}"
    
    # Client
    echo "📦 Client..."
    cd client
    if [ "$target" = "native" ]; then
        cargo build --release
    else
        cargo build --release --target "$target"
    fi
    cd ..
    
    # Serveur  
    echo "📦 Serveur..."
    cd serveur
    if [ "$target" = "native" ]; then
        cargo build --release
    else
        cargo build --release --target "$target"
    fi
    cd ..
    
    echo -e "${COLOR_GREEN}✅ Build $name réussi${COLOR_RESET}"
    echo
}

# Builds selon le paramètre
case $TARGET in
    windows)
        if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" || "$OSTYPE" == "win32" ]]; then
            build_target "native" "Windows natif"
        else
            build_target "x86_64-pc-windows-gnu" "Windows (cross)"
        fi
        ;;
    linux)
        if [[ "$OSTYPE" == "linux-gnu"* ]]; then
            build_target "native" "Linux natif"  
        else
            build_target "x86_64-unknown-linux-gnu" "Linux (cross)"
        fi
        ;;
    both)
        if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" || "$OSTYPE" == "win32" ]]; then
            build_target "native" "Windows natif"
            build_target "x86_64-unknown-linux-gnu" "Linux (cross)"
        else
            build_target "native" "Linux natif"
            build_target "x86_64-pc-windows-gnu" "Windows (cross)"
        fi
        ;;
    *)
        echo -e "${COLOR_RED}❌ Usage: $0 [windows|linux|both]${COLOR_RESET}"
        exit 1
        ;;
esac

echo -e "${COLOR_GREEN}🎉 Vérification cross-platform terminée avec succès !${COLOR_RESET}"