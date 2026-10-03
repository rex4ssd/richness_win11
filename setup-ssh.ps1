# setup-ssh.ps1
# 在 Win11 上把 Mac 的 SSH key 設定好，讓 git push/pull 不用輸入帳密
#
# 使用方式（推薦）：
#   1. 把這個檔跟 id_ed25519 / id_ed25519.pub 放在同一個資料夾（例如 USB 根目錄）
#   2. 在該資料夾開 PowerShell（Shift + 右鍵 → Open PowerShell here，或在 cmd 裡 cd 過去再打 powershell）
#   3. 執行：  powershell -ExecutionPolicy Bypass -File .\setup-ssh.ps1
#   4. 第一次需要管理員權限來把 ssh-agent 設成自動啟動，script 會偵測並提示
#
# 如果 key 不在同層，可指定路徑：
#   .\setup-ssh.ps1 -KeySource 'D:\ssh_mac'

[CmdletBinding()]
param(
    [string]$KeySource = $PSScriptRoot
)

$ErrorActionPreference = 'Stop'

function Step($msg) { Write-Host "`n==> $msg" -ForegroundColor Cyan }
function OK($msg)   { Write-Host "[OK] $msg"   -ForegroundColor Green }
function Warn2($msg){ Write-Host "[!!] $msg"   -ForegroundColor Yellow }

# 如果是雙擊執行，$PSScriptRoot 可能空字串
if ([string]::IsNullOrWhiteSpace($KeySource)) {
    $KeySource = (Get-Location).Path
}

# --- 0. 檢查來源檔 ---
$src_priv = Join-Path $KeySource 'id_ed25519'
$src_pub  = Join-Path $KeySource 'id_ed25519.pub'

if (-not (Test-Path $src_priv) -or -not (Test-Path $src_pub)) {
    Write-Host ""
    Write-Host "找不到 id_ed25519 / id_ed25519.pub 在：" -ForegroundColor Red
    Write-Host "  $KeySource" -ForegroundColor Red
    Write-Host ""
    Write-Host "請用 -KeySource 指定，例如：" -ForegroundColor Yellow
    Write-Host "  .\setup-ssh.ps1 -KeySource 'D:\ssh_mac'"
    exit 1
}
OK "找到 key 在: $KeySource"

# --- 1. 建立 .ssh 資料夾 ---
Step "建立 $env:USERPROFILE\.ssh"
$ssh_dir = Join-Path $env:USERPROFILE '.ssh'
if (-not (Test-Path $ssh_dir)) {
    New-Item -ItemType Directory -Path $ssh_dir | Out-Null
}
OK $ssh_dir

# --- 2. 複製 key ---
Step "複製 key 到 .ssh"
$dst_priv = Join-Path $ssh_dir 'id_ed25519'
$dst_pub  = Join-Path $ssh_dir 'id_ed25519.pub'
Copy-Item $src_priv $dst_priv -Force
Copy-Item $src_pub  $dst_pub  -Force
OK "id_ed25519 / id_ed25519.pub 已複製"

# --- 3. 設權限（Windows OpenSSH 要求私鑰只能自己讀，否則會拒用） ---
Step "設定私鑰權限"
icacls $dst_priv /inheritance:r | Out-Null
icacls $dst_priv /grant:r "$($env:USERNAME):(R)" | Out-Null
OK "權限 OK"

# --- 4. 寫 ~/.ssh/config（拿掉 macOS 專屬的 UseKeychain） ---
Step "寫 ~/.ssh/config"
$config_path = Join-Path $ssh_dir 'config'
$config = @"
Host github.com
  AddKeysToAgent yes
  IdentityFile ~/.ssh/id_ed25519
"@
# 用 ASCII 寫入避免 BOM 害到 ssh
Set-Content -Path $config_path -Value $config -Encoding ascii
OK "config OK"

# --- 5. 啟動 ssh-agent（設成 Automatic 需要管理員） ---
Step "啟動 ssh-agent 服務"
$is_admin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

if ($is_admin) {
    Set-Service ssh-agent -StartupType Automatic
    Start-Service ssh-agent
    OK "ssh-agent 已設為自動啟動並運行"
} else {
    Warn2 "目前不是管理員，無法把 ssh-agent 設成『自動啟動』。"
    Warn2 "等下請用『系統管理員 PowerShell』跑這兩行（一次就好）："
    Write-Host "    Set-Service ssh-agent -StartupType Automatic"
    Write-Host "    Start-Service ssh-agent"
    try { Start-Service ssh-agent -ErrorAction Stop; OK "本次已暫時啟動 ssh-agent" } catch {}
}

# --- 6. 加進 agent ---
Step "ssh-add"
try {
    ssh-add $dst_priv
    OK "key 已加入 agent"
} catch {
    Warn2 "ssh-add 失敗，可能是 ssh-agent 還沒啟動。先用管理員跑第 5 步再重試。"
}

# --- 7. 測試 GitHub 連線 ---
Step "測試 ssh -T git@github.com"
$test = & ssh -T -o StrictHostKeyChecking=accept-new git@github.com 2>&1
$test | ForEach-Object { Write-Host "    $_" }

if ($test -match "successfully authenticated") {
    Write-Host ""
    OK "搞定！現在可以："
    Write-Host "    git clone git@github.com:rex4ssd/richness_win11.git"
    Write-Host ""
    Warn2 "別忘了把 USB 裡的 id_ed25519 私鑰擦掉或鎖好（私鑰外流＝身份外流）"
} else {
    Warn2 "看起來沒成功。把上面 [==> 測試...] 那段輸出貼給 Claude 看一下。"
}
