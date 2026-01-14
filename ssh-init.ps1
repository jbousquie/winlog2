# Configuration PowerShell pour sessions SSH - Winlog 2
# Ce fichier configure automatiquement l'environnement MinGW pour les sessions SSH
#
# Usage :
#   . .\ssh-init.ps1
#
# Pour automatiser au démarrage, ajouter à votre profil PowerShell :
#   echo '. C:\path\to\winlog2\ssh-init.ps1' >> $PROFILE
#
# Voir WINDOWS_BUILD.md pour l'installation initiale de MSYS2 et la configuration permanente

# Détection de session SSH
if ($env:SSH_CLIENT -or $env:SSH_CONNECTION) {
    Write-Host "🔧 Configuration Winlog 2 pour session SSH..." -ForegroundColor Blue
    
    # Ajouter MSYS2 au PATH pour MinGW (dlltool, gcc, etc.)
    if (Test-Path "C:\msys64\mingw64\bin\dlltool.exe") {
        $msysPath = "C:\msys64\mingw64\bin"
        if (-not ($env:Path -like "*$msysPath*")) {
            $env:Path = $env:Path + ";$msysPath"
            Write-Host "✅ MSYS2 ajouté au PATH (session uniquement)" -ForegroundColor Green
        } else {
            Write-Host "ℹ️  MSYS2 déjà dans le PATH" -ForegroundColor DarkGray
        }
    } else {
        Write-Warning "⚠️  MSYS2 non trouvé. Installation nécessaire pour MinGW :"
        Write-Host "    winget install MSYS2.MSYS2" -ForegroundColor Yellow
        Write-Host "    Voir WINDOWS_BUILD.md pour les détails" -ForegroundColor Yellow
    }
    
    # Vérifier la toolchain Rust (informatif uniquement)
    $rustupShow = rustup show 2>$null
    if ($rustupShow -like "*stable-gnu*") {
        Write-Host "✅ Toolchain GNU active" -ForegroundColor Green
    } elseif ($rustupShow -like "*stable-msvc*") {
        Write-Host "ℹ️  Toolchain MSVC active (MSYS2 non nécessaire)" -ForegroundColor DarkGray
    } else {
        Write-Host "ℹ️  Toolchain: $($rustupShow -split "`n" | Select-Object -First 1)" -ForegroundColor DarkGray
    }
} else {
    Write-Host "ℹ️  Pas en session SSH - Configuration ignorée" -ForegroundColor DarkGray
    Write-Host "    Ce script est conçu pour les connexions SSH à distance" -ForegroundColor DarkGray
}