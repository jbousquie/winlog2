# Script de vérification et compilation pour Windows
# Usage : .\build-windows.ps1

Write-Host "=== Winlog 2 - Script de compilation Windows ===" -ForegroundColor Cyan
Write-Host ""

# 1. Vérification Rust
Write-Host "[1/5] Vérification de Rust..." -ForegroundColor Yellow
try {
    $rustVersion = rustc --version
    Write-Host "  ✓ Rust installé : $rustVersion" -ForegroundColor Green
} catch {
    Write-Host "  ✗ Rust non installé !" -ForegroundColor Red
    Write-Host "  Installer avec : winget install Rustlang.Rustup" -ForegroundColor Yellow
    exit 1
}

# 2. Vérification toolchain
Write-Host "[2/5] Vérification de la toolchain..." -ForegroundColor Yellow
$toolchain = rustup show active-toolchain
Write-Host "  ✓ Toolchain active : $toolchain" -ForegroundColor Green

# 3. Vérification du linker
Write-Host "[3/5] Vérification du linker..." -ForegroundColor Yellow
$linkerOk = $false

# Test MSVC
try {
    $null = Get-Command link.exe -ErrorAction Stop
    Write-Host "  ✓ MSVC linker détecté (link.exe)" -ForegroundColor Green
    $linkerOk = $true
} catch {
    Write-Host "  ⚠ MSVC linker non trouvé" -ForegroundColor DarkGray
}

# Test MinGW
if (-not $linkerOk) {
    try {
        $null = Get-Command gcc.exe -ErrorAction Stop
        Write-Host "  ✓ MinGW/GCC détecté (gcc.exe)" -ForegroundColor Green
        $linkerOk = $true
    } catch {
        Write-Host "  ⚠ MinGW/GCC non trouvé" -ForegroundColor DarkGray
    }
}

if (-not $linkerOk) {
    Write-Host "  ✗ Aucun linker détecté !" -ForegroundColor Red
    Write-Host "  Installez l'un des deux :" -ForegroundColor Yellow
    Write-Host "    - Visual Studio Build Tools : https://aka.ms/vs/17/release/vs_BuildTools.exe" -ForegroundColor Yellow
    Write-Host "    - MSYS2 + MinGW : winget install MSYS2.MSYS2" -ForegroundColor Yellow
    exit 1
}

# 4. Compilation du client
Write-Host "[4/5] Compilation du client..." -ForegroundColor Yellow
Push-Location client
try {
    cargo build --release 2>&1 | Out-Null
    if ($LASTEXITCODE -eq 0) {
        Write-Host "  ✓ Client compilé avec succès" -ForegroundColor Green
        Write-Host "    - target\release\logon.exe" -ForegroundColor DarkGray
        Write-Host "    - target\release\logout.exe" -ForegroundColor DarkGray
        Write-Host "    - target\release\matos.exe" -ForegroundColor DarkGray
    } else {
        Write-Host "  ✗ Erreur de compilation du client" -ForegroundColor Red
        Pop-Location
        exit 1
    }
} catch {
    Write-Host "  ✗ Erreur : $_" -ForegroundColor Red
    Pop-Location
    exit 1
}
Pop-Location

# 5. Compilation du serveur
Write-Host "[5/5] Compilation du serveur..." -ForegroundColor Yellow
Push-Location serveur
try {
    cargo build --release 2>&1 | Out-Null
    if ($LASTEXITCODE -eq 0) {
        Write-Host "  ✓ Serveur compilé avec succès" -ForegroundColor Green
        Write-Host "    - target\release\winlog-server.exe" -ForegroundColor DarkGray
    } else {
        Write-Host "  ✗ Erreur de compilation du serveur" -ForegroundColor Red
        Pop-Location
        exit 1
    }
} catch {
    Write-Host "  ✗ Erreur : $_" -ForegroundColor Red
    Pop-Location
    exit 1
}
Pop-Location

# Résumé final
Write-Host ""
Write-Host "=== Compilation terminée avec succès ! ===" -ForegroundColor Green
Write-Host ""
Write-Host "Binaires disponibles :" -ForegroundColor Cyan
Write-Host "  Client :" -ForegroundColor Yellow
Write-Host "    - client\target\release\logon.exe" -ForegroundColor White
Write-Host "    - client\target\release\logout.exe" -ForegroundColor White
Write-Host "    - client\target\release\matos.exe" -ForegroundColor White
Write-Host "  Serveur :" -ForegroundColor Yellow
Write-Host "    - serveur\target\release\winlog-server.exe" -ForegroundColor White
Write-Host ""
Write-Host "Prochaines étapes :" -ForegroundColor Cyan
Write-Host "  1. Configurer serveur\config.toml" -ForegroundColor White
Write-Host "  2. Créer la base SQLite : serveur\scripts\create_db.sh" -ForegroundColor White
Write-Host "  3. Démarrer le serveur : serveur\target\release\winlog-server.exe" -ForegroundColor White
Write-Host ""
