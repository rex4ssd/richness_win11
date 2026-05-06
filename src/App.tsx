// App.tsx — root layout
// Three hardware panels side-by-side, each identified by a host_id.
// Designed for easy N-host expansion: map over a hosts array.

import React, { useState } from "react";
import { RS485Panel } from "./components/RS485Panel";
import { USBPanel } from "./components/USBPanel";
import { RJ45Panel } from "./components/RJ45Panel";

// For multi-host scale: add more host IDs here.
const DEFAULT_HOST = "local";

export default function App() {
  const [hostId] = useState(DEFAULT_HOST);

  return (
    <div className="flex flex-col h-screen bg-gray-950 text-gray-100">
      {/* Header */}
      <header className="flex items-center justify-between px-4 py-2 bg-gray-900 border-b border-gray-800 shrink-0">
        <div className="flex items-center gap-3">
          <span className="text-base font-bold tracking-wider text-gray-100">
            HW Comm Tool
          </span>
          <span className="text-xs text-gray-500">RS-485 · USB · RJ45</span>
        </div>
        <div className="flex items-center gap-2 text-xs text-gray-500">
          <span>host:</span>
          <span className="font-mono text-gray-300">{hostId}</span>
        </div>
      </header>

      {/* Three panels — equal width, fill remaining height */}
      <main className="flex flex-1 min-h-0 gap-3 p-3">
        <div className="flex-1 min-w-0">
          <RS485Panel hostId={hostId} />
        </div>
        <div className="flex-1 min-w-0">
          <USBPanel hostId={hostId} />
        </div>
        <div className="flex-1 min-w-0">
          <RJ45Panel hostId={hostId} />
        </div>
      </main>

      {/* Status bar */}
      <footer className="px-4 py-1 bg-gray-900 border-t border-gray-800 text-xs text-gray-600 shrink-0">
        richness-win11 v0.1.0 · Tauri 2 · Windows 11 target
      </footer>
    </div>
  );
}
