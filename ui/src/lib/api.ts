import { invoke } from "@tauri-apps/api/core";
import type {
  CrossoverToggles,
  DohProvider,
  GosSettings,
  LeakReport,
  PrivacyStatus,
} from "./types";

// Thin, typed wrappers over the Rust commands. Arg keys match the Rust
// parameter names exactly (all single words, so no case translation needed).
export const api = {
  getStatus: () => invoke<PrivacyStatus>("get_status"),
  getSettings: () => invoke<GosSettings>("get_settings"),
  getNodeId: () => invoke<string>("get_node_id"),

  startTor: () => invoke<PrivacyStatus>("start_tor"),
  stopTor: () => invoke<PrivacyStatus>("stop_tor"),
  newIdentity: () => invoke<PrivacyStatus>("new_identity"),

  setDoh: (enabled: boolean, provider: DohProvider) =>
    invoke<PrivacyStatus>("set_doh", { enabled, provider }),

  runLeakTest: () => invoke<LeakReport>("run_leak_test"),

  setKillSwitch: (enabled: boolean) =>
    invoke<GosSettings>("set_kill_switch", { enabled }),
  setMacRandomize: (enabled: boolean) =>
    invoke<string>("set_mac_randomize", { enabled }),

  setCrossover: (toggles: CrossoverToggles) =>
    invoke<GosSettings>("set_crossover", { toggles }),

  listDir: (path: string) => invoke<unknown[]>("bridge_list_dir", { path }),
  launchApp: (name: string) => invoke<void>("bridge_launch_app", { name }),
  listInterfaces: () => invoke<string[]>("bridge_list_interfaces"),
  clipboardGet: () => invoke<string>("bridge_clipboard_get"),
  clipboardSet: (text: string) =>
    invoke<void>("bridge_clipboard_set", { text }),
  homeDir: () => invoke<string>("home_dir"),
};
