# How to Deploy — richness-win11

## 目標
Push 到 GitHub 後，GitHub Actions 自動編譯 Windows 11 的 `.exe` 安裝程式，
並上傳到 GitHub Releases 供下載。

---

## 1. 前置條件

### 本機開發 (macOS)
```bash
# 安裝 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安裝 Node.js 20+
brew install node

# 安裝 Tauri CLI
npm install  # 已在 package.json 中列為 devDependency

# 安裝 serialport 系統依賴 (macOS 開發用)
# serialport 在 macOS 上不需要額外安裝

# 安裝 libusb (macOS，只用於本機測試)
brew install libusb

# 啟動開發伺服器 (連前端 HMR)
npm run tauri dev
```

---

## 2. 上傳到 GitHub 並觸發自動編譯

### 步驟 A — 確認 remote
```bash
cd /Users/lion/Documents/richness_win11
git remote -v
# 應顯示 origin git@github.com:rex4ssd/richness_win11
```

### 步驟 B — 新增所有檔案並 commit
```bash
git add -A
git commit -m "feat: initial Tauri 2 hardware comm tool"
git push origin main
```

### 步驟 C — 打版本 Tag 觸發 Release 編譯
```bash
# 語意版本，例如 v0.1.0
git tag v0.1.0
git push origin v0.1.0
```

打完 tag 後，到 GitHub repo 的 **Actions** 頁面：
- 可以看到 `Build Windows Installer` workflow 自動執行
- 完成後在 **Releases** 頁出現 `richness-win11_0.1.0_x64-setup.exe`

---

## 3. 在 Windows 11 下載安裝

1. 到 GitHub repo → **Releases** → 最新版本
2. 下載 `richness-win11_x.y.z_x64-setup.exe`
3. 執行安裝（每個使用者安裝，無需管理員）
4. 從開始選單或桌面捷徑啟動

---

## 4. USB 設備額外設定 (Windows 11 必做)

libusb/rusb 需要裝置使用 **WinUSB** 驅動。

1. 下載 [Zadig](https://zadig.akeo.ie/)
2. 插入 USB 設備
3. Zadig → Options → List All Devices
4. 選擇你的設備 → Driver 選 `WinUSB` → Install Driver
5. 之後 USB 面板就可以正常列舉與通訊

---

## 5. Log 檔位置 (Windows 11)

```
C:\Users\<YourName>\AppData\Roaming\richness-win11\logs\
└── richness.YYYY-MM-DD   ← 每日輪替
```

---

## 6. 手動觸發 CI (不打 tag)

GitHub repo → Actions → `Build Windows Installer` → **Run workflow** → 選 `main`

---

## 7. 多主機擴充說明

目前 `host_id` 預設為 `"local"`。若要監控 10 台以上主機：

**前端 `App.tsx`**：
```tsx
const hosts = ["local", "192.168.1.10", "192.168.1.11", ...];
// 用 map 渲染多組 RS485Panel / USBPanel / RJ45Panel
```

**後端 `AppState`**：
- `hosts: Mutex<HashMap<String, HostState>>` 已設計為 N-host
- 每個 `host_id` 完全隔離，Start/Stop 互不影響
- 事件 payload 包含 `host_id`，前端可精確過濾

---

## 8. 常見問題

| 問題 | 解法 |
|------|------|
| CI 失敗：`libusb not found` | 確認 workflow 中 vcpkg 步驟執行成功 |
| USB 面板：`ACCESS_DENIED` | 執行 Zadig 安裝 WinUSB 驅動 |
| RS-485 找不到 COM port | 安裝設備的 USB-Serial 驅動（FTDI/CH340）|
| 安裝檔被 SmartScreen 擋 | 點「更多資訊」→「仍然執行」，或為 NSIS 簽章 |
| Log 沒輸出 | 確認 `%APPDATA%\richness-win11\logs\` 目錄可寫入 |
