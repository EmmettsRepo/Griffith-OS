import type { DohProvider, GosSettings, PrivacyStatus } from "../lib/types";
import { Switch, ToggleRow } from "./Toggle";

interface Props {
  status: PrivacyStatus | null;
  settings: GosSettings | null;
  busy: string | null;
  onToggleTor: () => void;
  onSetDoh: (enabled: boolean, provider?: DohProvider) => void;
  onSetKill: (enabled: boolean) => void;
  onSetMac: (enabled: boolean) => void;
}

export function PrivacyControls({
  status,
  settings,
  busy,
  onToggleTor,
  onSetDoh,
  onSetKill,
  onSetMac,
}: Props) {
  const dohOn = !!status?.dohEnabled;
  const provider = settings?.dohProvider ?? "cloudflare";

  return (
    <section className="card card-pad">
      <div className="card-head">
        <div className="card-title">Anonymity controls</div>
      </div>

      <div className="rows">
        <ToggleRow
          label="Tor routing"
          desc="Route through the Tor network and exit from a relay IP."
          on={!!status?.torEnabled}
          busy={busy === "tor"}
          onChange={onToggleTor}
        />

        <div className="row">
          <div className="row-text">
            <div className="label">Encrypted DNS (DoH)</div>
            <div className="desc">
              Resolve lookups over HTTPS so the network can't read them.
            </div>
          </div>
          <div style={{ display: "flex", alignItems: "center", gap: "0.6rem" }}>
            <select
              className="select"
              value={provider}
              disabled={busy === "doh"}
              onChange={(e) => onSetDoh(dohOn, e.target.value as DohProvider)}
            >
              <option value="cloudflare">Cloudflare</option>
              <option value="quad9">Quad9</option>
              <option value="google">Google</option>
            </select>
            <Switch
              label="Encrypted DNS"
              on={dohOn}
              busy={busy === "doh"}
              onChange={() => onSetDoh(!dohOn)}
            />
          </div>
        </div>

        <ToggleRow
          label="Kill switch"
          desc="Block all traffic if the tunnel drops (needs the privileged helper)."
          on={!!settings?.killSwitch}
          busy={busy === "kill"}
          onChange={() => onSetKill(!settings?.killSwitch)}
        />

        <ToggleRow
          label="Randomize MAC"
          desc="Change your hardware address on the local network (needs admin)."
          on={!!settings?.macRandomized}
          busy={busy === "mac"}
          onChange={() => onSetMac(!settings?.macRandomized)}
        />
      </div>
    </section>
  );
}
