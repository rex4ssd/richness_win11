// RS485Panel.tsx — RS-485 control panel

import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { HwDataEvent, HwLogEvent } from "../types";
import { useTerminal } from "../hooks/useTerminal";
import { TerminalView } from "./TerminalView";

interface Props {
  hostId: string;
}

export function RS485Panel({ hostId }: Props) {
  const { lines, push, clear, bottomRef } = useTerminal();

  const [ports, setPorts] = useState<string[]>([]);
  const [selectedPort, setSelectedPort] = useState("");
  const [baud, setBaud] = useState(9600);
  const [running, setRunning] = useState(false);
  const [sendText, setSendText] = useState("");
  const [error, setError] = useState<string | null>(null);

  // Load port list on mount and every time panel focuses
  useEffect(() => {
    refreshPorts();
  }, []);

  async function refreshPorts() {
    try {
      const list = await invoke<string[]>("rs485_list_ports");
      setPorts(list);
      if (list.length > 0 && !selectedPort) setSelectedPort(list[0]);
    } catch (e) {
      push("ERROR", `List ports: ${e}`);
    }
  }

  // Subscribe to Tauri events for this interface
  useEffect(() => {
    const unlisteners: UnlistenFn[] = [];

    listen<HwDataEvent>("rs485-data", (ev) => {
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
    if (!selectedPort) return;
    setError(null);
    try {
      await invoke("rs485_start", { hostId, port: selectedPort, baud });
      setRunning(true);
      push("INFO", `Started on ${selectedPort} @ ${baud} baud`);
    } catch (e: unknown) {
      const msg = String(e);
      setError(msg);
      push("ERROR", msg);
    }
  }

  async function handleStop() {
    try {
      await invoke("rs485_stop", { hostId });
      setRunning(false);
      push("INFO", "Stopped");
    } catch (e: unknown) {
      push("ERROR", String(e));
    }
  }

  async function handleSend() {
    if (!sendText.trim()) return;
    const bytes = Array.from(new TextEncoder().encode(sendText));
    try {
      await invoke("rs485_send", { hostId, data: bytes });
      push("INFO", `TX: ${sendText}`);
      setSendText("");
    } catch (e: unknown) {
      push("ERROR", String(e));
    }
  }

  return (
    <PanelCard title="RS-485" color="amber" running={running}>
      {/* Config row */}
      <div className="flex flex-wrap gap-2 mb-2">
        <select
          className="flex-1 min-w-[140px] bg-gray-800 border border-gray-600 rounded px-2 py-1 text-sm"
          value={selectedPort}
          onChange={(e) => setSelectedPort(e.target.value)}
          disabled={running}
        >
          {ports.map((p) => (
            <option key={p}>{p}</option>
          ))}
        </select>
        <select
          className="bg-gray-800 border border-gray-600 rounded px-2 py-1 text-sm"
          value={baud}
          onChange={(e) => setBaud(Number(e.target.value))}
          disabled={running}
        >
          {[9600, 19200, 38400, 57600, 115200].map((b) => (
            <option key={b}>{b}</option>
          ))}
        </select>
        <button
          onClick={refreshPorts}
          disabled={running}
          className="px-2 py-1 text-xs rounded bg-gray-700 hover:bg-gray-600 disabled:opacity-40"
        >
          ↻
        </button>
        <button
          onClick={running ? handleStop : handleStart}
          className={`px-4 py-1 text-sm font-medium rounded ${
            running
              ? "bg-red-600 hover:bg-red-500"
              : "bg-amber-600 hover:bg-amber-500"
          }`}
        >
          {running ? "Stop" : "Start"}
        </button>
      </div>

      {error && <p className="text-xs text-red-400 mb-2">{error}</p>}

      {/* Terminal */}
      <div className="flex-1 min-h-0 border border-gray-700 rounded overflow-hidden">
        <TerminalView lines={lines} bottomRef={bottomRef} onClear={clear} />
      </div>

      {/* Send row */}
      <div className="flex gap-2 mt-2">
        <input
          className="flex-1 bg-gray-800 border border-gray-600 rounded px-2 py-1 text-sm font-mono"
          placeholder="Text to send (UTF-8)…"
          value={sendText}
          onChange={(e) => setSendText(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && handleSend()}
          disabled={!running}
        />
        <button
          onClick={handleSend}
          disabled={!running || !sendText.trim()}
          className="px-3 py-1 text-sm rounded bg-amber-700 hover:bg-amber-600 disabled:opacity-40"
        >
          Send
        </button>
      </div>
    </PanelCard>
  );
}

// ── Shared card wrapper ────────────────────────────────────────────────────────

interface CardProps {
  title: string;
  color: "amber" | "sky" | "emerald";
  running: boolean;
  children: React.ReactNode;
}

const borderColor = {
  amber: "border-amber-600",
  sky: "border-sky-600",
  emerald: "border-emerald-600",
};
const dotColor = {
  amber: "bg-amber-400",
  sky: "bg-sky-400",
  emerald: "bg-emerald-400",
};

export function PanelCard({ title, color, running, children }: CardProps) {
  return (
    <div
      className={`flex flex-col h-full bg-gray-900 border rounded-lg p-3 ${borderColor[color]}`}
    >
      <div className="flex items-center gap-2 mb-3">
        <span
          className={`w-2 h-2 rounded-full ${
            running ? dotColor[color] : "bg-gray-600"
          }`}
        />
        <h2 className="font-semibold text-sm tracking-widest uppercase text-gray-300">
          {title}
        </h2>
      </div>
      {children}
    </div>
  );
}
