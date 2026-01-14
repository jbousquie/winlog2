# Guide de déploiement Winlog 2 - Configuration par variables d'environnement

Ce guide explique comment déployer et configurer les clients Winlog 2 en production en utilisant les **variables d'environnement** pour la configuration centralisée.

## 🎯 Vue d'ensemble

**Avantages de cette approche** :
- ✅ **Pas de recompilation** : Changement de configuration sans rebuild
- ✅ **Déploiement centralisé** : GPO Windows / Ansible Linux
- ✅ **Un seul binaire** : Même binaire pour tous les environnements (dev/preprod/prod)
- ✅ **Audit facilité** : Changements tracés dans GPO/logs système
- ✅ **Flexibilité** : Configuration différente par machine/groupe si nécessaire

## 📋 Variables d'environnement supportées

| Variable | Type | Défaut | Description |
|----------|------|--------|-------------|
| `WINLOG_SERVER_URL` | String | `http://127.0.0.1:3000/api/v1/events` | URL complète du serveur de monitoring |
| `WINLOG_TIMEOUT` | u64 | `30` | Timeout HTTP en secondes |
| `WINLOG_MAX_RETRIES` | u32 | `3` | Nombre de tentatives maximum |
| `WINLOG_RETRY_DELAY_MS` | u64 | `1000` | Délai entre retries (millisecondes) |
| `WINLOG_USER_AGENT` | String | `Winlog/0.1.0` | User-Agent des requêtes HTTP |

---

## 🪟 Déploiement Windows

### Option 1 : GPO (Group Policy Object) - Recommandé pour entreprise

#### Étape 1 : Créer la GPO de configuration

1. Ouvrir **Group Policy Management Console** (gpmc.msc)
2. Créer une nouvelle GPO : `Winlog - Configuration Client`
3. Éditer la GPO créée

#### Étape 2 : Configurer les variables d'environnement

1. Naviguer vers :  
   **Computer Configuration** > **Preferences** > **Windows Settings** > **Environment**

2. Clic droit > **New** > **Environment Variable**

3. Créer les variables suivantes :

**Variable 1 - URL du serveur** :
- **Action** : Create
- **Name** : `WINLOG_SERVER_URL`
- **Value** : `http://192.168.1.100:3000/api/v1/events` (remplacer par votre IP serveur)
- **User variable** : Non coché (= variable système)

**Variable 2 - Timeout (optionnel)** :
- **Action** : Create
- **Name** : `WINLOG_TIMEOUT`
- **Value** : `60`
- **User variable** : Non coché

**Variable 3 - Max Retries (optionnel)** :
- **Action** : Create
- **Name** : `WINLOG_MAX_RETRIES`
- **Value** : `5`
- **User variable** : Non coché

#### Étape 3 : Appliquer la GPO

1. Lier la GPO aux OUs concernées (ex: `OU=Workstations`)
2. Forcer la mise à jour sur un poste test :
   ```powershell
   gpupdate /force
   ```

#### Étape 4 : Vérifier la configuration

```powershell
# Vérifier que les variables sont bien configurées
Get-ChildItem Env:WINLOG_*

# Sortie attendue :
# Name                           Value
# ----                           -----
# WINLOG_SERVER_URL              http://192.168.1.100:3000/api/v1/events
# WINLOG_TIMEOUT                 60
# WINLOG_MAX_RETRIES             5
```

### Option 2 : PowerShell (test local ou déploiement scriptable)

#### Script de configuration automatique

```powershell
# deploy-winlog-config.ps1
# À exécuter en tant qu'administrateur

param(
    [Parameter(Mandatory=$true)]
    [string]$ServerUrl = "http://192.168.1.100:3000/api/v1/events",
    
    [int]$Timeout = 30,
    [int]$MaxRetries = 3,
    [int]$RetryDelayMs = 1000
)

Write-Host "=== Configuration Winlog 2 - Client ===" -ForegroundColor Cyan

# Configuration des variables d'environnement système
[System.Environment]::SetEnvironmentVariable("WINLOG_SERVER_URL", $ServerUrl, "Machine")
[System.Environment]::SetEnvironmentVariable("WINLOG_TIMEOUT", $Timeout.ToString(), "Machine")
[System.Environment]::SetEnvironmentVariable("WINLOG_MAX_RETRIES", $MaxRetries.ToString(), "Machine")
[System.Environment]::SetEnvironmentVariable("WINLOG_RETRY_DELAY_MS", $RetryDelayMs.ToString(), "Machine")

Write-Host "✓ Variables d'environnement configurées" -ForegroundColor Green
Write-Host "  - WINLOG_SERVER_URL: $ServerUrl" -ForegroundColor Gray
Write-Host "  - WINLOG_TIMEOUT: $Timeout" -ForegroundColor Gray
Write-Host "  - WINLOG_MAX_RETRIES: $MaxRetries" -ForegroundColor Gray
Write-Host "  - WINLOG_RETRY_DELAY_MS: $RetryDelayMs" -ForegroundColor Gray

Write-Host ""
Write-Host "⚠️  Redémarrer la session pour appliquer les changements" -ForegroundColor Yellow
Write-Host "   Ou exécuter : refreshenv (si Chocolatey installé)" -ForegroundColor Yellow
```

**Utilisation** :
```powershell
# Avec paramètres par défaut (prod)
.\deploy-winlog-config.ps1 -ServerUrl "http://192.168.1.100:3000/api/v1/events"

# Avec timeout personnalisé
.\deploy-winlog-config.ps1 -ServerUrl "http://10.0.0.50:3000/api/v1/events" -Timeout 60 -MaxRetries 5
```

### Option 3 : Configuration manuelle (test rapide)

```powershell
# Configuration système (persistante, nécessite droits admin)
[System.Environment]::SetEnvironmentVariable(
    "WINLOG_SERVER_URL", 
    "http://192.168.1.100:3000/api/v1/events", 
    "Machine"
)

# Configuration session (temporaire, pour la session PowerShell actuelle)
$env:WINLOG_SERVER_URL = "http://192.168.1.100:3000/api/v1/events"
```

### Déploiement des binaires Windows (GPO)

Une fois la configuration en place :

1. Copier les binaires vers SYSVOL :
   ```powershell
   Copy-Item logon.exe \\domain.local\SYSVOL\domain.local\scripts\winlog\
   Copy-Item logout.exe \\domain.local\SYSVOL\domain.local\scripts\winlog\
   Copy-Item matos.exe \\domain.local\SYSVOL\domain.local\scripts\winlog\
   ```

2. Configurer les scripts de session (GPO) :
   - **Computer Configuration** > **Windows Settings** > **Scripts (Startup/Shutdown)**
   - **User Configuration** > **Windows Settings** > **Scripts (Logon/Logoff)**
   
   Ajouter :
   - Logon : `\\domain.local\SYSVOL\domain.local\scripts\winlog\logon.exe`
   - Logoff : `\\domain.local\SYSVOL\domain.local\scripts\winlog\logout.exe`

---

## 🐧 Déploiement Linux

### Option 1 : /etc/environment (Recommandé - Ubuntu/Debian)

#### Configuration manuelle

```bash
# Éditer /etc/environment avec les privilèges root
sudo nano /etc/environment

# Ajouter les lignes suivantes :
WINLOG_SERVER_URL=http://192.168.1.100:3000/api/v1/events
WINLOG_TIMEOUT=30
WINLOG_MAX_RETRIES=3
WINLOG_RETRY_DELAY_MS=1000
```

**Recharger l'environnement** :
```bash
# Pour la session actuelle
source /etc/environment

# Ou se reconnecter
logout
```

#### Script de déploiement automatique

```bash
#!/bin/bash
# deploy-winlog-config.sh

SERVER_URL="${1:-http://192.168.1.100:3000/api/v1/events}"
TIMEOUT="${2:-30}"
MAX_RETRIES="${3:-3}"
RETRY_DELAY_MS="${4:-1000}"

echo "=== Configuration Winlog 2 - Client ==="

# Vérifier les droits root
if [ "$EUID" -ne 0 ]; then 
    echo "❌ Ce script doit être exécuté en tant que root"
    exit 1
fi

# Backup de /etc/environment
cp /etc/environment /etc/environment.backup.$(date +%Y%m%d%H%M%S)

# Supprimer anciennes entrées Winlog (si existantes)
sed -i '/^WINLOG_/d' /etc/environment

# Ajouter nouvelles variables
cat >> /etc/environment <<EOF
WINLOG_SERVER_URL=$SERVER_URL
WINLOG_TIMEOUT=$TIMEOUT
WINLOG_MAX_RETRIES=$MAX_RETRIES
WINLOG_RETRY_DELAY_MS=$RETRY_DELAY_MS
EOF

echo "✓ Configuration ajoutée à /etc/environment"
echo "  - WINLOG_SERVER_URL: $SERVER_URL"
echo "  - WINLOG_TIMEOUT: $TIMEOUT"
echo "  - WINLOG_MAX_RETRIES: $MAX_RETRIES"
echo "  - WINLOG_RETRY_DELAY_MS: $RETRY_DELAY_MS"
echo ""
echo "⚠️  Les utilisateurs doivent se reconnecter pour appliquer les changements"
```

**Utilisation** :
```bash
sudo ./deploy-winlog-config.sh http://192.168.1.100:3000/api/v1/events
```

### Option 2 : /etc/profile.d (Alternative - CentOS/RHEL)

```bash
#!/bin/bash
# Créer le script de configuration

sudo cat > /etc/profile.d/winlog.sh <<'EOF'
# Configuration Winlog 2 Client
export WINLOG_SERVER_URL=http://192.168.1.100:3000/api/v1/events
export WINLOG_TIMEOUT=30
export WINLOG_MAX_RETRIES=3
export WINLOG_RETRY_DELAY_MS=1000
EOF

# Rendre le script exécutable
sudo chmod +x /etc/profile.d/winlog.sh

# Charger immédiatement
source /etc/profile.d/winlog.sh
```

### Option 3 : Ansible (Déploiement massif)

```yaml
# playbook-winlog-config.yml
---
- name: Configure Winlog 2 Client
  hosts: all
  become: yes
  vars:
    winlog_server_url: "http://192.168.1.100:3000/api/v1/events"
    winlog_timeout: 30
    winlog_max_retries: 3
    winlog_retry_delay_ms: 1000
  
  tasks:
    - name: Add Winlog environment variables to /etc/environment
      lineinfile:
        path: /etc/environment
        regexp: "^{{ item.key }}="
        line: "{{ item.key }}={{ item.value }}"
        state: present
      loop:
        - { key: "WINLOG_SERVER_URL", value: "{{ winlog_server_url }}" }
        - { key: "WINLOG_TIMEOUT", value: "{{ winlog_timeout }}" }
        - { key: "WINLOG_MAX_RETRIES", value: "{{ winlog_max_retries }}" }
        - { key: "WINLOG_RETRY_DELAY_MS", value: "{{ winlog_retry_delay_ms }}" }
      notify: Inform users to re-login

  handlers:
    - name: Inform users to re-login
      debug:
        msg: "Environment variables updated. Users must re-login to apply changes."
```

**Exécution** :
```bash
ansible-playbook -i inventory.ini playbook-winlog-config.yml
```

### Déploiement des binaires Linux (PAM)

```bash
# Copier les binaires
sudo cp logon logout matos /usr/local/bin/
sudo chmod +x /usr/local/bin/{logon,logout,matos}

# Configurer PAM pour logon (à l'ouverture de session)
echo '/usr/local/bin/logon &' | sudo tee /etc/profile.d/winlog-logon.sh
sudo chmod +x /etc/profile.d/winlog-logon.sh

# Configurer pour logout (à la fermeture)
echo '/usr/local/bin/logout &' >> ~/.bash_logout

# Tâche cron pour inventaire matériel (2h du matin)
(crontab -l 2>/dev/null; echo "0 2 * * * /usr/local/bin/matos") | crontab -
```

---

## 🧪 Tests de validation

### Test de configuration

#### Windows
```powershell
# Vérifier variables d'environnement
Get-ChildItem Env:WINLOG_*

# Tester un binaire avec configuration personnalisée
$env:WINLOG_SERVER_URL = "http://test.local:3000/api/v1/events"
.\matos.exe
```

#### Linux
```bash
# Vérifier variables d'environnement
env | grep WINLOG

# Tester un binaire avec configuration personnalisée
WINLOG_SERVER_URL=http://test.local:3000/api/v1/events ./matos
```

### Test de connectivité

```bash
# Test réseau vers le serveur
curl -X POST http://192.168.1.100:3000/api/v1/events \
  -H "Content-Type: application/json" \
  -d '{"username":"test","action":"C","timestamp":"2024-01-01T00:00:00Z"}'
```

---

## 🔧 Dépannage

### Windows : Variables non chargées

**Symptôme** : Le client utilise toujours `127.0.0.1:3000`

**Solutions** :
1. Vérifier la GPO :
   ```powershell
   gpresult /H gpo-report.html
   # Ouvrir gpo-report.html et vérifier "Applied GPOs"
   ```

2. Forcer la mise à jour :
   ```powershell
   gpupdate /force
   ```

3. Vérifier après redémarrage de session :
   ```powershell
   Get-ChildItem Env:WINLOG_*
   ```

### Linux : Variables non chargées dans cron

**Problème** : Les variables de `/etc/environment` ne sont pas chargées dans les tâches cron.

**Solution** : Spécifier explicitement dans la crontab :
```bash
0 2 * * * export $(cat /etc/environment | grep WINLOG | xargs) && /usr/local/bin/matos
```

Ou créer un wrapper :
```bash
#!/bin/bash
# /usr/local/bin/matos-wrapper.sh
source /etc/environment
/usr/local/bin/matos
```

---

## 📚 Documentation complémentaire

- **Configuration client détaillée** : `client/README.md`
- **Architecture globale** : `README.md`
- **Scripts serveur** : `serveur/scripts/README.md`

---

## ✅ Checklist de déploiement

### Windows
- [ ] GPO créée avec variables d'environnement
- [ ] GPO liée aux OUs concernées
- [ ] `gpupdate /force` exécuté sur poste test
- [ ] Variables visibles : `Get-ChildItem Env:WINLOG_*`
- [ ] Binaires déployés sur SYSVOL
- [ ] Scripts logon/logoff configurés via GPO
- [ ] Test d'exécution manuelle réussi

### Linux
- [ ] Variables ajoutées à `/etc/environment` ou `/etc/profile.d/winlog.sh`
- [ ] Backup de configuration créé
- [ ] Variables visibles après reconnexion : `env | grep WINLOG`
- [ ] Binaires copiés dans `/usr/local/bin/`
- [ ] Permissions exécutables définies (`chmod +x`)
- [ ] PAM/profile.d configuré pour logon
- [ ] Cron configuré pour matos
- [ ] Test d'exécution manuelle réussi
