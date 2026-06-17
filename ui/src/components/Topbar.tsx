import type { ConnectionState, PrivacyStatus } from "../lib/types";
import { GlobeIcon, ShieldIcon } from "./icons";

const STATE_LABEL: Record<ConnectionState, string> = {
  connected: "Connected",
  connecting: "Connecting",
  disconnected: "Offline",
  error: "Error",
};

export function Topbar({ status }: { status: PrivacyStatus | null }) {
  const state = status?.state ?? "disconnected";
  return (
    <header className="topbar">
      <div className="brand">
        <div className="brand-mark">
          <ShieldIcon size={18} />
        </div>
        <div>
          <div className="brand-name">GRIFFITH OS</div>
          <div className="brand-sub">privacy extension · v0.1</div>
        </div>
      </div>
      <div className="topbar-right">
        {status?.exitCountry && (
          <span className="pill">
            <GlobeIcon />
            {status.exitCountry}
          </span>
        )}
        <span className="pill live">
          <span className={"dot" + (state === "connecting" ? " pulse" : "")} />
          {STATE_LABEL[state]}
        </span>
      </div>
    </header>
  );
}
