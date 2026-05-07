// USBPanel.tsx — USB control panel with device dropdown

import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { HwDataEvent, HwLogEvent, UsbDeviceInfo } from "../types";
import { useTerminal } from "../hooks/useTerminal";
import { TerminalView } from "./TerminalView";
import { PanelCard } from "./RS485Panel";

interface Props {
  hostId: string;
}

export function USBPanel({ hostId }: Props) {
  const { lines, push, clear, bottomRef } = useTerminal();

  const [devices, setDevices] = useState<UsbDeviceInfo[]>([]);
  const [selectedIdx, setSelectedIdx] = useState(0);
  const [interfaceNum, setInterfaceNum] = useState(0);
  const [running, setRunning] = useState(false);
  const [sendHex, setSendHex] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    refreshDevices();
  }, []);

  async function refreshDevices() {
    setLoading(true);
    try {
      const list = await invoke<UsbDeviceInfo[]>("usb_list_devices");
      setDevices(list);
    } catch (e) {
      push("ERROR", `List USB: ${e}`);
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    const unlisteners: UnlistenFn[] = [];

    listen<HwDataEvent>("usb-data", (ev) => {
      if (ev.payload.host_id !== hostId) return;
      push("DATA", `[${ev.payload.raw_len}B] ${ev.payload.hex}`, ev.payload.timestamp);
    }).then((u) => unlisteners.push(u));

    listen<HwLogEvent>("hw-log", (ev) => {
      if (!ev.payload.message.includes(`[${hostId}]`)) return;
      push(ev.payload.level as "INFO" | "WARN" | "ERROR", ev.payload.message, ev.payload.timestamp);
    }).then((u) => unlisteners.push(u));

    return () => unlisteners.forEach((u) => u());
  }, [hostId, push]);

  async function handleStart() {
    const dev = devices[selectedIdx];
    if (!dev) return;
    setError(null);
    try {
      await invoke("usb_start", {
        hostId,
        vid: dev.vid,
        pid: dev.pid,
        interfaceNum,
      });
      setRunning(true);
      push("INFO", `Started VID=${dev.vid.toString(16).padStart(4,"0")} PID=${dev.pid.toString(16).padStart(4,"0")}`);
    } catch (e: unknown) {
      const msg = String(e);
      setError(msg);
      push("ERROR", msg);
    }
  }

  async function handleStop() {
    try {
      await invoke("usb_stop", { hostId });
      setRunning(false);
      push("INFO", "Stopped");
    } catch (e: unknown) {
      push("ERROR", String(e));
    }
  }

  async function handleSend() {
    // Parse hex string like "DE AD BE EF"
    const clean = sendHex.replace(/\s+/g, "");
    if (!clean || clean.length % 2 !== 0) {
      push("WARN", "Invalid hex — must be even number of hex chars");
      return;
    }
    const bytes = [];
    for (let i = 0; i < clean.length; i += 2) {
      bytes.push(parseInt(clean.slice(i, i + 2), 16));
    }
    try {
      await invoke("usb_send", { hostId, endpoint: 0, data: bytes });
      push("INFO", `TX: ${sendHex}`);
      setSendHex("");
    } catch (e: unknown) {
      push("ERROR", String(e));
    }
  }

  const dev = devices[selectedIdx];

  return (
    <PanelCard title="USB" color="sky" running={running}>
      {/* Device selector */}
      <div className="flex flex-wrap gap-2 mb-2">
        <select
          className="flex-1 min-w-[180px] bg-gray-800 border border-gray-600 rounded px-2 py-1 text-xs"
          value={selectedIdx}
          onChange={(e) => setSelectedIdx(Number(e.target.value))}
          disabled={running}
        >
          {devices.length === 0 && (
            <option value={0}>— no devices found —</option>
          )}
          {devices.map((d, i) => (
            <option key={d.bus_address} value={i}>
              {d.description}
            </option>
          ))}
        </select>
        <div className="flex items-center gap-1 text-xs">
          <span className="text-gray-400">IF:</span>
          <input
            type="number"
            min={0}
            max={127}
            value={interfaceNum}
            onChange={(e) => setInterfaceNum(Number(e.target.value))}
            disabled={running}
            className="w-12 bg-gray-800 border border-gray-600 rounded px-1 py-1 text-center"
          />
        </div>
        <button
          onClick={refreshDevices}
          disabled={running || loading}
          className="px-2 py-1 text-xs rounded bg-gray-700 hover:bg-gray-600 disabled:opacity-40"
        >
          {loading ? "…" : "↻"}
        </button>
        <button
          onClick={running ? handleStop : handleStart}
          disabled={devices.length === 0 && !running}
          className={`px-4 py-1 text-sm font-medium rounded ${
            running
              ? "bg-red-600 hover:bg-red-500"
              : "bg-sky-600 hover:bg-sky-500"
          } disabled:opacity-40`}
        >
          {running ? "Stop" : "Start"}
        </button>
      </div>

      {dev && !running && (
        <p className="text-xs text-gray-500 mb-1">
          VID={dev.vid.toString(16).padStart(4,"0")} PID={dev.pid.toString(16).padStart(4,"0")} · {dev.bus_address}
        </p>
      )}
      {error && <p className="text-xs text-red-400 mb-2">{error}</p>}

      {/* Terminal */}
      <div className="flex-1 min-h-0 border border-gray-700 rounded overflow-hidden">
        <TerminalView lines={lines} bottomRef={bottomRef} onClear={clear} />
      </div>

      {/* Send row (hex input) */}
      <div className="flex gap-2 mt-2">
        <input
          className="flex-1 bg-gray-800 border border-gray-600 rounded px-2 py-1 text-sm font-mono"
          placeholder="Hex bytes to send, e.g. DE AD BE EF"
          value={sendHex}
          onChange={(e) => setSendHex(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && handleSend()}
          disabled={!running}
        />
        <button
          onClick={handleSend}
          disabled={!running || !sendHex.trim()}
          className="px-3 py-1 text-sm rounded bg-sky-700 hover:bg-sky-600 disabled:opacity-40"
        >
          Send
        </button>
      </div>
    </PanelCard>
  );
}
