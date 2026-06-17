interface SwitchProps {
  on: boolean;
  busy?: boolean;
  label: string;
  onChange: () => void;
}

export function Switch({ on, busy, label, onChange }: SwitchProps) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={on}
      aria-label={label}
      className="switch"
      data-on={on}
      data-busy={busy ?? false}
      onClick={onChange}
    >
      <span className="knob" />
    </button>
  );
}

interface ToggleRowProps {
  label: string;
  desc?: string;
  on: boolean;
  busy?: boolean;
  onChange: () => void;
}

export function ToggleRow({ label, desc, on, busy, onChange }: ToggleRowProps) {
  return (
    <div className="row">
      <div className="row-text">
        <div className="label">{label}</div>
        {desc && <div className="desc">{desc}</div>}
      </div>
      <Switch on={on} busy={busy} label={label} onChange={onChange} />
    </div>
  );
}
