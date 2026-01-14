# Configuration PowerShell pour sessions SSH - Winlog 2
# Ce fichier doit être sourcé dans le profil PowerShell pour les sessions SSH

# Ajouter MSYS2 au PATH pour avoir accès aux outils MinGW (dlltool, gcc, etc.)
# Nécessaire pour compiler avec la toolchain stable-x86_64-pc-windows-gnu
if ($env:SSH_CLIENT -or $env:SSH_CONNECTION) {
    Write-Host "🔧 Configuration Winlog 2 pour session SSH..." -ForegroundColor Blue
    
    # Vérifier si MSYS2 est installé
    if (Test-Path "C:\msys64\mingw64\bin\dlltool.exe") {
        $msysPath = "C:\msys64\mingw64\bin"
        if (-not ($env:Path -like "*$msysPath*")) {
            $env:Path = $env:Path + ";$msysPath"
            Write-Host "✅ MSYS2 ajouté au PATH" -ForegroundColor Green
        }
    } else {
        Write-Warning "⚠️  MSYS2 non trouvé. Installation nécessaire pour compiler Winlog 2 :"
        Write-Host "winget install MSYS2.MSYS2" -ForegroundColor Yellow
    }
    
    # Vérifier la toolchain Rust
    $rustupShow = rustup show 2>$null
    if ($rustupShow -like "*stable-x86_64-pc-windows-gnu*") {
        Write-Host "✅ Toolchain GNU configurée" -ForegroundColor Green
    } else {
        Write-Warning "⚠️  Toolchain recommandée : stable-x86_64-pc-windows-gnu"
    }
}