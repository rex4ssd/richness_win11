// RJ45Panel.tsx — RJ45 (TCP/UDP) control panel

import React, { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { HwDataEvent, HwLogEvent } from "../types";
import { useTerminal } from "../hooks/useTerminal";
import { TerminalView } from "./TerminalView";
import { PanelCard } from "./RS485Panel";

interface Props {
  hostId: string;
}

type Mode = "tcp_client" | "tcp_server" | "udp";

export function RJ45Panel({ hostId }: Props) {
  const { lines, push, clear, bottomRef } = useTerminal();

  const [mode, setMode] = useState<Mode>("tcp_client");
  const [addr, setAddr] = useState("127.0.0.1:8080");
  const [remoteAddr, setRemoteAddr] = useState("");
  const [running, setRunning] = useState(false);
  const [sendText, setSendText] = useState("");
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const unlisteners: UnlistenFn[] = [];

    listen<HwDataEvent>("rj45-data", (ev) => {
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
    setError(null);
    try {
      await invoke("rj45_start", {
        hostId,
        mode,
        addr,
        remoteAddr: remoteAddr || "",
      });
      setRunning(true);
      push("INFO", `Started ${mode} on ${addr}`);
    } catch (e: unknown) {
      const msg = String(e);
      setError(msg);
      push("ERROR", msg);
    }
  }

  async function handleStop() {
    try {
      await invoke("rj45_stop", { hostId });
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
      await invoke("rj45_send", { hostId, data: bytes });
      push("INFO", `TX: ${sendText}`);
      setSendText("");
    } catch (e: unknown) {
      push("ERROR", String(e));
    }
  }

  const addrPlaceholder =
    mode === "tcp_server" ? "0.0.0.0:8080" : "192.168.1.10:8080";

  return (
    <PanelCard title="RJ45 / Network" color="emerald" running={running}>
      {/* Config row */}
      <div className="flex flex-wrap gap-2 mb-2">
        <select
          className="bg-gray-800 border border-gray-600 rounded px-2 py-1 text-sm"
          value={mode}
          onChange={(e) => setMode(e.target.value as Mode)}
          disabled={running}
        >
          <option value="tcp_client">TCP Client</option>
          <option value="tcp_server">TCP Server</option>
          <option value="udp">UDP</option>
        </select>
        <input
          className="flex-1 min-w-[160px] bg-gray-800 border border-gray-600 rounded px-2 py-1 text-sm font-mono"
          placeholder={addrPlaceholder}
          value={addr}
          onChange={(e) => setAddr(e.target.value)}
          disabled={running}
        />
        {mode === "udp" && (
          <input
            className="flex-1 min-w-[140px] bg-gray-800 border border-gray-600 rounded px-2 py-1 text-sm font-mono"
            placeholder="UDP remote (host:port)"
            value={remoteAddr}
            onChange={(e) => setRemoteAddr(e.target.value)}
            disabled={running}
          />
        )}
        <button
          onClick={running ? handleStop : handleStart}
          className={`px-4 py-1 text-sm font-medium rounded ${
            running
              ? "bg-red-600 hover:bg-red-500"
              : "bg-emerald-600 hover:bg-emerald-500"
          }`}
        >
          {running ? "Stop" : "Connect"}
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
          placeholder="Message to send (UTF-8)…"
          value={sendText}
          onChange={(e) => setSendText(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && handleSend()}
          disabled={!running}
        />
        <button
          onClick={handleSend}
          disabled={!running || !sendText.trim()}
          className="px-3 py-1 text-sm rounded bg-emerald-700 hover:bg-emerald-600 disabled:opacity-40"
        >
          Send
        </button>
      </div>
    </PanelCard>
  );
}
