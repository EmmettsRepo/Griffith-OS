// Mirrors the serde types in gos-core (all camelCase across the IPC boundary).

export type ConnectionState =
  | "disconnected"
  | "connecting"
  | "connected"
  | "error";

export type LeakVerdict = "protected" | "partial" | "exposed";

export type DohProvider = "cloudflare" | "quad9" | "google";

export interface PrivacyStatus {
  state: ConnectionState;
  torEnabled: boolean;
  dohEnabled: boolean;
  vpnEnabled: boolean;
  killSwitch: boolean;
  macRandomized: boolean;
  exitIp: string | null;
  exitCountry: string | null;
  socksPort: number | null;
  nodeId: string;
  message: string | null;
}

export interface CrossoverToggles {
  files: boolean;
  launchApps: boolean;
  clipboard: boolean;
  devices: boolean;
}

export interface GosSettings {
  torEnabled: boolean;
  dohEnabled: boolean;
  vpnEnabled: boolean;
  killSwitch: boolean;
  macRandomized: boolean;
  dohProvider: DohProvider;
  crossover: CrossoverToggles;
  themeAccent: string;
}

export interface LeakReport {
  realIp: string | null;
  exitIp: string | null;
  ipMasked: boolean;
  dohActive: boolean;
  verdict: LeakVerdict;
  details: string[];
  checkedAtMs: number;
}
