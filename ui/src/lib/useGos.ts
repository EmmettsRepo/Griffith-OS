import { useCallback, useEffect, useState } from "react";
import { api } from "./api";
import type {
  CrossoverToggles,
  DohProvider,
  GosSettings,
  LeakReport,
  PrivacyStatus,
} from "./types";

const POLL_MS = 6000;

export function useGos() {
  const [status, setStatus] = useState<PrivacyStatus | null>(null);
  const [settings, setSettings] = useState<GosSettings | null>(null);
  const [leak, setLeak] = useState<LeakReport | null>(null);
  const [busy, setBusy] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    try {
      const [s, set] = await Promise.all([api.getStatus(), api.getSettings()]);
      setStatus(s);
      setSettings(set);
    } catch (e) {
      setError(String(e));
    }
  }, []);

  useEffect(() => {
    refresh();
    const id = setInterval(() => {
      api.getStatus().then(setStatus).catch(() => {});
    }, POLL_MS);
    return () => clearInterval(id);
  }, [refresh]);

  // Run an action under a busy key, surfacing errors and refreshing after.
  const run = useCallback(
    async (key: string, fn: () => Promise<unknown>) => {
      setBusy(key);
      setError(null);
      try {
        await fn();
        await refresh();
      } catch (e) {
        setError(String(e));
      } finally {
        setBusy(null);
      }
    },
    [refresh],
  );

  const actions = {
    toggleTor: () =>
      run("tor", () => (status?.torEnabled ? api.stopTor() : api.startTor())),
    newIdentity: () => run("identity", () => api.newIdentity()),
    setDoh: (enabled: boolean, provider?: DohProvider) =>
      run("doh", () =>
        api.setDoh(enabled, provider ?? settings?.dohProvider ?? "cloudflare"),
      ),
    setKillSwitch: (enabled: boolean) =>
      run("kill", () => api.setKillSwitch(enabled)),
    setMac: (enabled: boolean) => run("mac", () => api.setMacRandomize(enabled)),
    setCrossover: (toggles: CrossoverToggles) =>
      run("crossover", () => api.setCrossover(toggles)),
    leakTest: () =>
      run("leak", async () => {
        const report = await api.runLeakTest();
        setLeak(report);
      }),
  };

  return { status, settings, leak, busy, error, actions };
}
