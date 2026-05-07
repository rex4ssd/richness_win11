# Role
你是一位資深的 Rust 與 Tauri 2.0 專家，專注於 Windows 11 平台的底層硬體通訊軟體開發。請保持程式碼極度簡潔、模組化，並嚴格處理所有非同步與錯誤邊界。
# Task
在目前的 Tauri 2.0 專案中，實作一個硬體通訊測試工具。
目標平台：Windows 11 (開發環境為 macOS，請注意路徑與 OS 條件編譯)。
# Tech Stack & Dependencies
- 前端：TypeScript, React (或 Vue), TailwindCSS
- 後端：Rust, Tauri 2.0
- 核心 Crate：`tokio` (非同步), `serialport` (RS-485), `rusb` (USB), `tracing` & `tracing-appender` (日誌)
# Core Requirements
1. **日誌系統 (Logging)**
   - 初始化 `tracing`，將 Log 同時輸出到 Console 與 Windows 的 AppData 目錄下的檔案 (每日輪替 rolling file)。
   - 包含 Info, Warn, Error 層級。
2. **後端硬體模組 (Rust)**
   - **RS-485**: 實作開啟/關閉 COM Port，並在背景 `tokio::spawn` 監聽資料。
   - **USB**: 實作列舉可用 USB 設備 (VID/PID 或 Port)，實作 Bulk In/Out 端點的讀寫。
   - **RJ45**: 實作 TCP Client/Server 或 UDP Socket 的連線與收發。
   - 所有接收到的資料，統一透過 Tauri IPC 事件 (e.g., `app_handle.emit`) 推播給前端。
3. **前端 GUI (UI/UX)**
   - 建立 3 個獨立的控制區塊：RS-485、USB、RJ45。
   - **USB 區塊**需包含一個下拉選單 (Dropdown)，可動態取得並選擇當前可用的 USB 設備/Port。
   - 每個區塊都有獨立的「Start / Stop」按鈕。
   - 按下 Start 後，下方需有一個 Terminal 風格的 Textarea 即時滾動顯示收到的訊號與 Log。
   - 實作「送出訊號」的 Input 欄位與 Send 按鈕。
# Coding Style Rules (Strict)
1. 禁用 `unwrap()` 與 `expect()`，所有硬體錯誤必須透過 `Result` 回傳並記錄至 Log。
2. 使用 `tauri::State` 與 `tokio::sync::Mutex` 管理連線狀態，確保 Start/Stop 切換時不會發生資源洩漏 (Resource Leak) 或 Thread Panic。
3. 提供結構清晰的專案目錄，前端與後端邏輯必須解耦。
請直接給我 `src-tauri/Cargo.toml` 依賴設定、後端 `[main.rs](http://main.rs)` 及硬體通訊模組的骨架代碼，以及前端的主頁面結構。
```</T,>

#開發完，在howto.md說明，如何放到github上，且github會自動complier .exe，讓我在windows11上下載
#github repo = [git@github.com](mailto:git@github.com):rex4ssd/richness_win11 ,已經設定好
#開發過程需參考 /Users/lion/Documents/richness_win11/coding_[style.md](http://style.md)(不用100分遵守)
#這是個demo tool，需考慮以後變大成需收/傳 10個台以上監控系統主機的訊號