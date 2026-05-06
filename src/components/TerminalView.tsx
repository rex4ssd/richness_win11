// TerminalView.tsx — scrollable terminal-style log display

import React from "react";
import { TerminalLine } from "../types";

interface Props {
  lines: TerminalLine[];
  bottomRef: React.RefObject<HTMLDivElement>;
  onClear: () => void;
}

const levelColor: Record<TerminalLine["level"], string> = {
  DATA: "text-green-400",
  INFO: "text-blue-300",
  WARN: "text-yellow-300",
  ERROR: "text-red-400",
};

export function TerminalView({ lines, bottomRef, onClear }: Props) {
  return (
    <div className="flex flex-col h-full">
      {/* toolbar */}
      <div className="flex justify-between items-center px-2 py-1 bg-gray-800 border-b border-gray-700">
        <span className="text-xs text-gray-400">
          {lines.length} line{lines.length !== 1 ? "s" : ""}
        </span>
        <button
          onClick={onClear}
          className="text-xs text-gray-400 hover:text-white px-2 py-0.5 rounded hover:bg-gray-700"
        >
          Clear
        </button>
      </div>

      {/* log area */}
      <div className="flex-1 overflow-y-auto terminal-scroll bg-gray-950 p-2 font-mono text-xs leading-5">
        {lines.map((line) => (
          <div key={line.id} className="flex gap-2">
            <span className="text-gray-600 shrink-0">
              {line.ts.slice(11, 23)}
            </span>
            <span className={`shrink-0 w-10 ${levelColor[line.level]}`}>
              {line.level}
            </span>
            <span className="text-gray-200 break-all">{line.text}</span>
          </div>
        ))}
        <div ref={bottomRef} />
      </div>
    </div>
  );
}
