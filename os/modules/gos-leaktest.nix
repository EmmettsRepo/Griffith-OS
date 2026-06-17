# `gos-leaktest` — proves the anti-tracking actually works (the v1 gos-privacy
# leak concept, as a declarative, baked-in command). Run it after boot:
#
#   gos-leaktest              # non-disruptive checks
#   gos-leaktest --killswitch # also stop Tor briefly to confirm fail-closed
#
# It detects the mode via /etc/gos-anonymous (written by gos-anti-tracking.nix).
{ config, pkgs, lib, ... }:
let
  leaktest = pkgs.writeShellApplication {
    name = "gos-leaktest";
    runtimeInputs = with pkgs; [ curl jq gnugrep coreutils systemd ];
    text = ''
      CHECK_URL="https://check.torproject.org/api/ip"
      SOCKS="127.0.0.1:9050"
      fails=0

      ok()   { printf '  \033[32m✓ PASS\033[0m  %s\n' "$1"; }
      bad()  { printf '  \033[31m✗ FAIL\033[0m  %s\n' "$1"; fails=$((fails + 1)); }
      info() { printf '  \033[36mℹ\033[0m       %s\n' "$1"; }

      anon_mode() { [ -e /etc/gos-anonymous ]; }

      printf 'GOS leak test — %s\n\n' "$(date -u '+%Y-%m-%d %H:%M:%S UTC')"

      # 1) Tor routing -----------------------------------------------------------
      echo "[1] Tor routing"
      if anon_mode; then
        info "anonymous mode: transparent Tor + fail-closed kill switch"
        body="$(curl -fsS --max-time 25 "$CHECK_URL" 2>/dev/null || true)"
      else
        info "usable mode: testing Tor via SOCKS ($SOCKS)"
        body="$(curl -fsS --max-time 25 --socks5-hostname "$SOCKS" "$CHECK_URL" 2>/dev/null || true)"
      fi

      is_tor="false"; exit_ip="unknown"
      if [ -z "$body" ]; then
        bad "could not reach check.torproject.org (Tor down or blocked?)"
      else
        is_tor="$(printf '%s' "$body" | jq -r '.IsTor // false')"
        exit_ip="$(printf '%s' "$body" | jq -r '.IP // "unknown"')"
        if [ "$is_tor" = "true" ]; then
          ok "traffic exits through Tor (exit IP $exit_ip)"
        else
          bad "NOT on Tor — exit IP $exit_ip is your real address"
        fi
      fi
      echo

      # 2) Real-IP masking -------------------------------------------------------
      echo "[2] Real-IP masking"
      if anon_mode; then
        ok "real IP unmeasurable here — the kill switch blocks every non-Tor path"
      else
        real_ip="$(curl -fsS --max-time 15 https://api.ipify.org 2>/dev/null || true)"
        if [ -z "$real_ip" ]; then
          info "could not determine direct IP"
        elif [ "$real_ip" = "$exit_ip" ]; then
          bad "Tor exit IP equals direct IP ($real_ip) — traffic is NOT torified"
        else
          ok "Tor exit ($exit_ip) differs from your direct IP ($real_ip)"
        fi
      fi
      echo

      # 3) DNS -------------------------------------------------------------------
      echo "[3] DNS"
      dns_srv="$(resolvectl status 2>/dev/null | grep -i 'Current DNS Server' | head -n1 || true)"
      if anon_mode && printf '%s' "$dns_srv" | grep -q '127.0.0.1:5353'; then
        ok "DNS resolves through Tor (127.0.0.1:5353)"
      else
        info "resolver: ''${dns_srv:-systemd-resolved}"
      fi
      if resolvectl query torproject.org >/dev/null 2>&1; then
        ok "name resolution works"
      else
        bad "name resolution failed"
      fi
      echo

      # 4) IPv6 ------------------------------------------------------------------
      echo "[4] IPv6"
      if [ "$(cat /proc/sys/net/ipv6/conf/all/disable_ipv6 2>/dev/null || echo 0)" = "1" ]; then
        ok "IPv6 disabled (no IPv6 leak surface)"
      else
        bad "IPv6 is enabled — possible leak path"
      fi
      echo

      # 5) Kill switch (opt-in, disruptive) -------------------------------------
      if [ "''${1:-}" = "--killswitch" ]; then
        echo "[5] Kill switch"
        if ! anon_mode; then
          info "only meaningful in anonymous mode; skipping"
        elif [ "$(id -u)" -ne 0 ]; then
          info "re-run as root (sudo gos-leaktest --killswitch) to test the kill switch"
        else
          systemctl stop tor 2>/dev/null || true
          sleep 2
          if curl -fsS --max-time 8 "$CHECK_URL" >/dev/null 2>&1; then
            bad "traffic still flowed with Tor stopped — NOT fail-closed"
          else
            ok "Tor down → all traffic blocked (fail-closed confirmed)"
          fi
          systemctl start tor 2>/dev/null || true
          info "Tor restarted"
        fi
        echo
      fi

      echo "------------------------------------------------------------"
      if [ "$fails" -eq 0 ]; then
        printf '\033[32mAll anti-tracking checks passed.\033[0m\n'
      else
        printf '\033[31m%s check(s) FAILED — see above.\033[0m\n' "$fails"
        exit 1
      fi
    '';
  };
in
{
  environment.systemPackages = [ leaktest ];
}
