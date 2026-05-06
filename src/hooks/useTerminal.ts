// useTerminal.ts — manage a capped ring-buffer of terminal lines

import { useCallback, useRef, useState } from "react";
import { TerminalLine } from "../types";

const MAX_LINES = 1000;
let idCounter = 0;

export function useTerminal() {
  const [lines, setLines] = useState<TerminalLine[]>([]);
  const bottomRef = useRef<HTMLDivElement | null>(null);

  const push = useCallback(
    (level: TerminalLine["level"], text: string, ts?: string) => {
      const line: TerminalLine = {
        id: idCounter++,
        ts: ts ?? new Date().toISOString(),
        level,
        text,
      };
      setLines((prev) => {
        const next = [...prev, line];
        // keep ring buffer bounded
        return next.length > MAX_LINES ? next.slice(next.length - MAX_LINES) : next;
      });
      // auto-scroll
      setTimeout(() => {
        bottomRef.current?.scrollIntoView({ behavior: "smooth" });
      }, 0);
    },
    []
  );

  const clear = useCallback(() => setLines([]), []);

  return { lines, push, clear, bottomRef };
}
