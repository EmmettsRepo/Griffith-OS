import type { CrossoverToggles, GosSettings } from "../lib/types";
import { ToggleRow } from "./Toggle";

interface Props {
  settings: GosSettings | null;
  busy: string | null;
  onChange: (toggles: CrossoverToggles) => void;
}

const ALL_ON: CrossoverToggles = {
  files: true,
  launchApps: true,
  clipboard: true,
  devices: true,
};
const ALL_OFF: CrossoverToggles = {
  files: false,
  launchApps: false,
  clipboard: false,
  devices: false,
};

export function CrossoverPanel({ settings, busy, onChange }: Props) {
  const c = settings?.crossover ?? ALL_ON;
  const isolated = !c.files && !c.launchApps && !c.clipboard && !c.devices;
  const working = busy === "crossover";
  const set = (patch: Partial<CrossoverToggles>) => onChange({ ...c, ...patch });

  return (
    <section className="card card-pad">
      <div className="card-head">
        <div className="card-title">Host cross-over</div>
      </div>

      <div className="iso-banner" data-iso={isolated}>
        <div className="row-text">
          <div className="label">Isolation mode</div>
          <div className="desc">Cut every bridge to the host at once.</div>
        </div>
        <button
          className="btn"
          disabled={working}
          onClick={() => onChange(isolated ? ALL_ON : ALL_OFF)}
        >
          {isolated ? "Reconnect host" : "Isolate now"}
        </button>
      </div>

      <div className="rows">
        <ToggleRow
          label="Files & folders"
          desc="Browse and open host files from GOS."
          on={c.files}
          busy={working}
          onChange={() => set({ files: !c.files })}
        />
        <ToggleRow
          label="Launch host apps"
          desc="Open installed host apps from GOS."
          on={c.launchApps}
          busy={working}
          onChange={() => set({ launchApps: !c.launchApps })}
        />
        <ToggleRow
          label="Clipboard sync"
          desc="Share copy / paste with the host."
          on={c.clipboard}
          busy={working}
          onChange={() => set({ clipboard: !c.clipboard })}
        />
        <ToggleRow
          label="Devices & network"
          desc="Use host Wi-Fi, camera, mic, printers."
          on={c.devices}
          busy={working}
          onChange={() => set({ devices: !c.devices })}
        />
      </div>
    </section>
  );
}
