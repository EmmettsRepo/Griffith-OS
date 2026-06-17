import { useGos } from "./lib/useGos";
import { Topbar } from "./components/Topbar";
import { StatusHero } from "./components/StatusHero";
import { PrivacyControls } from "./components/PrivacyControls";
import { LeakPanel } from "./components/LeakPanel";
import { CrossoverPanel } from "./components/CrossoverPanel";
import { NodeCard } from "./components/NodeCard";

export default function App() {
  const { status, settings, leak, busy, error, actions } = useGos();

  return (
    <div
      className="app"
      data-state={status?.state ?? "disconnected"}
      data-verdict={leak?.verdict}
    >
      <Topbar status={status} />

      <main className="main">
        {error && <div className="err">{error}</div>}

        <StatusHero
          status={status}
          busy={busy}
          onToggleTor={actions.toggleTor}
          onNewIdentity={actions.newIdentity}
        />

        <div className="grid">
          <div className="col">
            <PrivacyControls
              status={status}
              settings={settings}
              busy={busy}
              onToggleTor={actions.toggleTor}
              onSetDoh={actions.setDoh}
              onSetKill={actions.setKillSwitch}
              onSetMac={actions.setMac}
            />
            <LeakPanel leak={leak} busy={busy} onRun={actions.leakTest} />
          </div>

          <div className="col">
            <CrossoverPanel
              settings={settings}
              busy={busy}
              onChange={actions.setCrossover}
            />
            <NodeCard nodeId={status?.nodeId ?? ""} />
          </div>
        </div>
      </main>
    </div>
  );
}
