# 🪟 Guide de compilation Windows - Winlog 2

Ce guide explique comment compiler le projet Winlog 2 sur Windows avec les deux toolchains supportées : **MSVC** (recommandé) ou **MinGW/GNU**.

## 🎯 Choix de la toolchain

Le projet utilise `rust-toolchain.toml` avec `channel = "stable"`, ce qui permet à Rust de s'adapter automatiquement à votre environnement Windows.

### Option 1 : MSVC (recommandé - plus simple) ⭐

**Avantages** :
- Installation simple via Visual Studio Build Tools
- Binaires natifs Windows optimisés
- Compatibilité maximale avec l'écosystème Windows
- Aucune configuration PATH nécessaire

**Inconvénients** :
- Téléchargement plus volumineux (~1-2 GB)

### Option 2 : MinGW/GNU (pour uniformité cross-platform)

**Avantages** :
- Même toolchain que Linux (facilite le développement cross-platform)
- Installation plus légère via MSYS2

**Inconvénients** :
- Configuration PATH nécessaire
- Peut nécessiter des ajustements pour certaines dépendances

---

## 🚀 Installation et compilation

### Option 1 : Avec MSVC (recommandé)

#### 1. Installer les prérequis

```powershell
# Installer Rust (si pas déjà fait)
winget install Rustlang.Rustup

# Installer Visual Studio Build Tools (choisir "Desktop development with C++")
# Télécharger depuis : https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022
```

Ou via Chocolatey :
```powershell
choco install rust visualstudio2022buildtools
```

#### 2. Vérifier l'installation

```powershell
rustc --version
cargo --version
```

#### 3. Compiler le projet

```powershell
# Cloner le projet
git clone <url-du-repo>
cd winlog2

# Compiler le client
cd client
cargo build --release

# Compiler le serveur
cd ../serveur
cargo build --release
```

**C'est tout !** Les binaires sont dans `target/release/`.

---

### Option 2 : Avec MinGW/GNU

#### 1. Installer MSYS2

```powershell
# Via winget (recommandé)
winget install MSYS2.MSYS2

# Ou via Chocolatey
choco install msys2
```

#### 2. Installer les outils MinGW

Ouvrir un terminal **MSYS2 MSYS** et exécuter :

```bash
# Mettre à jour les paquets
pacman -Syu

# Installer la toolchain MinGW
pacman -S mingw-w64-x86_64-gcc mingw-w64-x86_64-toolchain
```

#### 3. Configurer Rust pour MinGW

```powershell
# Installer la toolchain GNU
rustup toolchain install stable-gnu

# Définir GNU comme toolchain par défaut
rustup default stable-gnu

# Vérifier
rustup show
```

#### 4. Configurer le PATH

**Temporaire (session PowerShell actuelle)** :
```powershell
$env:Path += ";C:\msys64\mingw64\bin"
```

**Permanent (recommandé)** :
```powershell
[Environment]::SetEnvironmentVariable(
    "Path",
    $env:Path + ";C:\msys64\mingw64\bin",
    "User"
)

# Redémarrer PowerShell pour appliquer les changements
```

#### 5. Compiler le projet

```powershell
# Cloner le projet
git clone <url-du-repo>
cd winlog2

# Compiler le client
cd client
cargo build --release

# Compiler le serveur
cd ../serveur
cargo build --release
```

---

## 🔍 Vérification de la compilation

### Vérifier les binaires générés

```powershell
# Client
dir client\target\release\logon.exe
dir client\target\release\logout.exe
dir client\target\release\matos.exe

# Serveur
dir serveur\target\release\winlog-server.exe
```

### Tester l'exécution

```powershell
# Test du serveur (nécessite config.toml configuré)
cd serveur
.\target\release\winlog-server.exe

# Test client (nécessite serveur démarré)
cd ..\client
.\target\release\matos.exe
```

---

## 🐛 Dépannage

### Erreur : "linker 'link.exe' not found"

**Cause** : Visual Studio Build Tools ou MinGW non installé

**Solution MSVC** :
- Installer [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022)
- Sélectionner "Desktop development with C++"

**Solution MinGW** :
- Suivre les étapes de l'Option 2 ci-dessus

### Erreur : "cannot find -lgcc" ou "ld.exe not found"

**Cause** : PATH MinGW non configuré (Option 2)

**Solution** :
```powershell
# Vérifier que MinGW est dans le PATH
$env:Path -split ';' | Select-String mingw

# Si vide, ajouter :
$env:Path += ";C:\msys64\mingw64\bin"
```

### Erreur : "rustup override set" ne fonctionne pas

**Cause** : Le projet n'utilise plus `rustup override` mais `rust-toolchain.toml`

**Solution** : Aucune action nécessaire, le fichier `rust-toolchain.toml` gère automatiquement la configuration.

### Sessions SSH : PATH non chargé

**Cause** : Les variables d'environnement utilisateur ne sont pas toujours chargées dans les sessions SSH

**Solution** :

**Option 1 - Utiliser le script fourni (recommandé)** :
```powershell
# Le projet inclut un script prêt à l'emploi
. .\ssh-init.ps1
```

**Option 2 - Configuration manuelle** :
```powershell
# Dans chaque session SSH, ajouter manuellement :
$env:Path += ";C:\msys64\mingw64\bin"
```

**Option 3 - Automatisation permanente** :
```powershell
# Ajouter au profil PowerShell pour charger automatiquement
echo '. C:\path\to\winlog2\ssh-init.ps1' >> $PROFILE
```

---

## 📊 Comparaison des toolchains

| Critère | MSVC | MinGW/GNU |
|---------|------|-----------|
| Installation | Build Tools (1-2 GB) | MSYS2 (~500 MB) |
| Configuration | Automatique | PATH manuel |
| Performance | Native Windows | Légèrement moins optimisé |
| Compatibilité cross-platform | Moyenne | Excellente |
| Recommandation | ✅ Production Windows | ✅ Développement cross-platform |

---

## 📚 Ressources

- [Rust sur Windows - Documentation officielle](https://doc.rust-lang.org/book/ch01-01-installation.html#installing-rustup-on-windows)
- [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022)
- [MSYS2 Documentation](https://www.msys2.org/)
- [Rustup Book - Toolchains](https://rust-lang.github.io/rustup/concepts/toolchains.html)

---

## ✅ Checklist de compilation

- [ ] Rust installé (`rustc --version`)
- [ ] Toolchain choisie (MSVC ou MinGW)
- [ ] Build Tools ou MSYS2 installé
- [ ] PATH configuré (pour MinGW uniquement)
- [ ] `cargo build --release` réussit
- [ ] Binaires générés dans `target/release/`
- [ ] Tests d'exécution OK

**Tout fonctionne ?** 🎉 Vous pouvez maintenant déployer les binaires !
