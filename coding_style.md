1. Rust 後端規範 (效能與穩定性)
零 unwrap() 與 expect() 政策：Tauri Command 若發生 Panic 會直接導致整個應用程式崩潰。必須嚴格使用 Result<T, E> 返回錯誤。

統一錯誤處理 (thiserror + anyhow)：
底層硬體模組使用 thiserror 定義具體的錯誤型別（例如 UsbError, SerialTimeout）。
傳遞給前端的 Command 統一回傳 Result<T, String> 或實作了 serde::Serialize 的自定義錯誤。
狀態與併發管理：硬體通訊的 Handle（如 Serial Port 連線）必須存放在 Tauri 的全局狀態 tauri::State 中。跨執行緒存取時，嚴格使用 tokio::sync::Mutex 或 std::sync::RwLock，避免死結 (Deadlock)。


2. Tauri IPC (前後端通訊) 架構
胖後端、瘦前端：所有邏輯運算、硬體狀態機、資料解析都在 Rust 完成。前端 (Vue/React) 只做單純的資料渲染與觸發。
事件驅動取代輪詢 (Polling)：對於 RS-485 或 USB 持續進來的訊號，絕對不要讓前端設定 setInterval 去狂 call Rust Command。
正確作法：Rust 端開一個 tokio::spawn 背景任務監聽硬體，收到資料後透過 app_handle.emit("hw-data-received", payload) 主動推播給前端。
Raw Bytes 傳輸：遇到大容量的二進位資料（如更新 SSD FW 的 bin 檔），避免轉成 Base64 或 JSON 陣列傳輸，利用 Tauri 2.0 的 IPC Raw Buffer 傳遞，降低 CPU 序列化開銷。