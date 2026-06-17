interface Props {
  nodeId: string;
}

export function NodeCard({ nodeId }: Props) {
  const short = nodeId ? `${nodeId.slice(0, 10)}…${nodeId.slice(-6)}` : "—";

  return (
    <section className="card card-pad">
      <div className="card-head">
        <div className="card-title">This install</div>
      </div>
      <div className="kv" style={{ borderTop: "none", paddingTop: 0 }}>
        <span className="k">Node ID</span>
        <span className="v" title={nodeId}>
          {short}
        </span>
      </div>
      <p className="note" style={{ marginTop: "0.8rem" }}>
        Your node ID identifies this install for encrypted GOS-to-GOS transfer
        (coming next). GOS targets strong, Tails-grade privacy — not literal
        invisibility. Browser fingerprinting, logged-in accounts, and global
        traffic correlation can still identify you.
      </p>
    </section>
  );
}
