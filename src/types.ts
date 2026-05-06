// types.ts — shared TypeScript types mirroring Rust structs

export interface HwDataEvent {
  host_id: string;
  interface: "rs485" | "usb" | "rj45";
  timestamp: string;
  hex: string;
  raw_len: number;
}

export interface HwLogEvent {
  level: "INFO" | "WARN" | "ERROR";
  message: string;
  timestamp: string;
}

export interface UsbDeviceInfo {
  vid: number;
  pid: number;
  bus_address: string;
  description: string;
}

export type TerminalLine = {
  id: number;
  ts: string;
  level: "DATA" | "INFO" | "WARN" | "ERROR";
  text: string;
};
