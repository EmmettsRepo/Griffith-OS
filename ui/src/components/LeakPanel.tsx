import type { LeakReport } from "../lib/types";

interface Props {
  leak: LeakReport | null;
  busy: string | null;
  onRun: () => void;
}

export function LeakPanel({ leak, busy, onRun }: Props) {
  return (
    <section className="card card-pad">
      <div className="card-head">
        <div className="card-title">Leak test</div>
        <button className="btn" onClick={onRun} disabled={busy === "leak"}>
          {busy === "leak" ? "Testing…" : "Run test"}
        </button>
      </div>

      {!leak ? (
        <p className="note">
          Compare your real IP against your exit IP and confirm DNS is
          encrypted. The verdict is honest — it never claims "untrackable".
        </p>
      ) : (
        <>
          <div
            style={{
              display: "flex",
              alignItems: "center",
              gap: "0.75rem",
              marginBottom: "0.5rem",
            }}
          >
            <span className="verdict" data-v={leak.verdict}>
              {leak.verdict.toUpperCase()}
            </span>
            <span className="note">
              {new Date(leak.checkedAtMs).toLocaleTimeString()}
            </span>
          </div>
          <div className="kv">
            <span className="k">Real IP</span>
            <span className="v">{leak.realIp ?? "—"}</span>
          </div>
          <div className="kv">
            <span className="k">Exit IP</span>
            <span className="v">{leak.exitIp ?? "—"}</span>
          </div>
          <div className="kv">
            <span className="k">DNS encrypted</span>
            <span className="v">{leak.dohActive ? "yes" : "no"}</span>
          </div>
          <ul className="detail-list">
            {leak.details.map((d, i) => (
              <li key={i}>{d}</li>
            ))}
          </ul>
        </>
      )}
    </section>
  );
}
