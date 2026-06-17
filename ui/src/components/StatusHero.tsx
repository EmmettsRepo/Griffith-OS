import type { ConnectionState, PrivacyStatus } from "../lib/types";
import { ShieldIcon } from "./icons";

const COPY: Record<ConnectionState, { word: string; msg: string }> = {
  connected: {
    word: "PROTECTED",
    msg: "Traffic is routing through Tor. Your real IP is hidden behind the exit relay.",
  },
  connecting: { word: "CONNECTING", msg: "Bootstrapping a Tor circuit…" },
  disconnected: {
    word: "EXPOSED",
    msg: "No tunnel is active. Your real IP is visible to the sites you visit.",
  },
  error: { word: "ERROR", msg: "The privacy engine reported a problem." },
};

interface Props {
  status: PrivacyStatus | null;
  busy: string | null;
  onToggleTor: () => void;
  onNewIdentity: () => void;
}

export function StatusHero({ status, busy, onToggleTor, onNewIdentity }: Props) {
  const state = status?.state ?? "disconnected";
  const copy = COPY[state];
  const torOn = !!status?.torEnabled;

  return (
    <section className="card hero">
      <div>
        <div className="hero-eyebrow">Privacy status</div>
        <h1 className="hero-state">{copy.word}</h1>
        <p className="hero-msg">{status?.message || copy.msg}</p>

        <div className="hero-ips">
          <div className="hero-ip">
            <div className="k">Exit IP</div>
            <div className={"v" + (status?.exitIp ? " masked" : "")}>
              {status?.exitIp ?? "—"}
              {status?.exitCountry ? ` · ${status.exitCountry}` : ""}
            </div>
          </div>
        </div>

        <div style={{ display: "flex", gap: "0.6rem", marginTop: "1.3rem" }}>
          <button
            className="btn btn-primary"
            onClick={onToggleTor}
            disabled={busy === "tor"}
          >
            {busy === "tor"
              ? "Working…"
              : torOn
                ? "Disconnect Tor"
                : "Connect Tor"}
          </button>
          <button
            className="btn btn-ghost"
            onClick={onNewIdentity}
            disabled={!torOn || busy === "identity"}
          >
            {busy === "identity" ? "Rotating…" : "New identity"}
          </button>
        </div>
      </div>

      <div className="shield" aria-hidden="true">
        <ShieldIcon />
      </div>
    </section>
  );
}
