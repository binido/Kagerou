# Roadmap

What works today, what is half-built, and what is planned. This file is the
single source of truth for feature status — if a row here disagrees with the
README or a comment in the code, this file is right and the other one is stale.

Kagerou's functional target is [NekoBox for
Android](https://github.com/MatsuriDayo/NekoBoxForAndroid): the same feature
set, on the desktop, with a UI that doesn't hurt. Where a NekoBox feature only
makes sense on Android, it's marked `Won't do` with the reason, and any desktop
equivalent is listed as its own row. The [parity table](#parity-with-nekobox-for-android)
at the end maps NekoBox's surface onto these sections.

## Status legend

| | Meaning |
|---|---|
| ✅ | Done — implemented and working. |
| 🟡 | Partial — visible in the app but incomplete or non-functional behind the UI. The note says what's missing. |
| 📋 | Planned — not started. The note says what it involves. |
| ❌ | Won't do — with the reason. |

Rows marked **Good first issue** are self-contained: small, well-bounded, and
they don't require understanding the whole app. Start there.

## Keeping this file honest

A pull request that changes behaviour updates its row in the same commit. A
`📋` becoming a `✅` is part of the change, not a follow-up. `🟡` exists
specifically so that half-finished work is visible instead of quietly reading
as done — do not upgrade a row to `✅` because the UI looks right.

See [CONTRIBUTING.md](CONTRIBUTING.md#picking-something-up) for how to claim a
row.

---

## Core & connectivity

| Feature | Status | Notes |
|---|---|---|
| Bundled sing-box core | ✅ | Version-pinned (1.14.0), sha256-verified, fetched at build time, shipped as a Tauri sidecar. |
| Start / stop the core | ✅ | Supervisor with crash and unexpected-exit detection, capped log ring buffer. |
| Config generation | ✅ | Built from stored profiles and rules: mixed inbound, one outbound per profile, a `proxy` selector, `clash_api`. |
| Clash API client | ✅ | Version, proxies, connections, outbound selection, connection closing, traffic websocket with auto-reconnect. |
| Hot profile switch while connected | ✅ | Selecting a profile switches the selector through the Clash API instead of restarting the core. |
| TUN mode | 🟡 | Config generation and per-OS privilege elevation (UAC / admin prompt / `CAP_NET_ADMIN` or `pkexec`) are implemented and unit-tested, but no end-to-end run against a live server has been done yet. Needs verification before it can be called done. |
| System proxy | 🟡 | The dashboard toggle is frontend-only state. Nothing sets the OS proxy: no `networksetup` (macOS), registry write (Windows), or GSettings/environment handling (Linux). |
| Inbound listen address and port | 📋 | The mixed inbound is hardcoded to `127.0.0.1:2080` and the Clash API to `127.0.0.1:9090`. Both should be settings, along with an "allow LAN access" toggle that binds `0.0.0.0`. **Good first issue.** |
| Configurable log level | 📋 | The generated config hardcodes `"level": "info"`. NekoBox exposes this in settings. **Good first issue.** |
| Configurable connection-test URL | 📋 | Delay tests hardcode `http://www.gstatic.com/generate_204`. **Good first issue.** |
| TUN tuning: MTU, stack, IPv6 mode | 📋 | Hardcoded to `stack: system`, `172.19.0.1/30`, no MTU or IPv6 handling. The existing `tunInterface` setting is stored but never read by the config generator — either wire it up or drop it. |
| Auto-connect on launch | 📋 | Connect to the last active profile on startup, gated by a setting. |
| Reset connections on network change / wake | 📋 | sing-box holds stale connections after a network switch or a laptop resume. NekoBox has both as toggles. |
| Custom config override | 📋 | Let a user append or override parts of the generated sing-box JSON, globally and per profile. Escape hatch for anything the UI doesn't expose. |

## Profiles & subscriptions

| Feature | Status | Notes |
|---|---|---|
| Protocols: VLESS, VMess, Trojan, Shadowsocks, Hysteria2, TUIC | ✅ | Parsed from links and generated into sing-box outbounds. |
| Subscription formats | ✅ | Plain or base64 URI lists, Clash-style YAML (`proxies:`), sing-box JSON (`outbounds`). |
| Profile groups | ✅ | Create, rename, move profiles between groups, reorder, drag-and-drop. |
| Subscription sources | ✅ | Add by URL or pasted key, manual refresh, remove. |
| Per-profile delay test | ✅ | TCP ping and URL test through the Clash API, colour-coded results. |
| Sort profiles by ping / name / protocol | ✅ | Setting applied per group. |
| Protocols: SOCKS, HTTP(S), WireGuard, SSH, ShadowTLS, AnyTLS | 📋 | All supported by the sing-box core; each needs a link/config parser and an outbound builder. Roughly one PR per protocol. |
| Manual profile creation | 📋 | Profiles can only be added by pasting a link or config. NekoBox has a per-protocol form (server, port, transport, TLS, mux, fingerprint…). This is the largest missing piece in this section and should be split per protocol. |
| Subscription auto-update | 🟡 | `autoUpdateSubscriptions` and its interval are stored in settings and shown in the UI, but nothing schedules a refresh. Needs a background task honouring the interval. |
| Subscription options: custom User-Agent, deduplication, force-resolve | 📋 | NekoBox exposes all three per group. Deduplication is the cheapest and most useful. |
| Group-wide delay test | 📋 | Test every profile in a group, then "delete unavailable" and "clear results" actions. |
| Export and sharing | 📋 | Copy a profile as a link, show it as a QR code, export a whole group to clipboard or file. |
| QR code import | 📋 | Scan a QR from an image file, the clipboard, or a screen region. |
| Backup and restore | 📋 | Export groups, profiles, routing rules, and settings as one JSON file, and import it back. NekoBox lets you pick which of the three to include. |
| Auto-select fastest (urltest group) | 📋 | Groups currently generate a plain selector. sing-box's `urltest` outbound gives automatic failover. |
| Proxy chains | 📋 | Route one proxy through another. NekoBox has both a chain profile type and per-group front/landing proxies. |
| Per-profile traffic statistics | 📋 | Bytes moved per profile, persisted, with a "clear statistics" action. |

## Routing & DNS

| Feature | Status | Notes |
|---|---|---|
| Routing rules | 🟡 | A rule is a single match string plus an outbound, classified into `domain` / `domain_suffix` / `ip_cidr` by shape. NekoBox's rules also carry port, source, source port, network, and protocol; sing-box supports all of them. The storage schema needs to grow before the UI can. |
| Routing presets | 🟡 | `Bypass LAN` and `Block ads` are stored, toggle in the UI, and are then ignored — the config generator never reads them. Either generate the corresponding rules or remove the section. **Good first issue.** |
| DNS | 📋 | The generated config has no `dns` section at all, so sing-box falls back to its defaults. NekoBox exposes remote and direct DNS servers, per-scope domain strategies, DNS routing, and FakeDNS. Leaks live here; this is the highest-priority row in this section. |
| Traffic sniffing | 📋 | No `sniff` on the inbounds, so domain-based rules can't match TLS/HTTP traffic arriving as an IP. Small change, large effect on whether routing rules work at all. |
| geoip / geosite rule sets | 📋 | Rules can't reference `geosite:category-ads` or `geoip:cn`. Needs rule-set support plus asset download and update, which NekoBox has as a separate "route assets" screen. |
| Per-process routing | 📋 | The desktop equivalent of NekoBox's per-app proxy: route by process name or path. sing-box supports `process_name` and `process_path` on Windows, macOS, and Linux. |
| Rule import / export | 📋 | Share rule sets as files, and ship a few sane defaults. |

## UI & UX

| Feature | Status | Notes |
|---|---|---|
| Dashboard, groups, sources, routing, logs, settings | ✅ | Six pages, all driven by the real backend. |
| Themes | ✅ | Catppuccin and Kanagawa flavours. |
| Localisation | ✅ | English and Russian. |
| Live traffic telemetry | ✅ | Speeds and session totals from sing-box's own traffic and connections endpoints, so they survive a frontend reload. |
| Log viewer | ✅ | Streams the core's output live, with level detection. |
| Traffic chart scale | 🟡 | `TelemetryChart` hardcodes `YAxis domain={[0, 100]}` while the points hold raw bytes/second, so any real traffic clips the line flat against the top. Fix by rescaling the axis or converting the series to Mbps in the store. **Good first issue.** |
| Connection list | 📋 | The Clash API already reports every live connection (host, rule, upload, download, duration); nothing displays them. Include "close connection" and "close all". |
| Embedded sing-box dashboard | 📋 | NekoBox bundles Yacd. The Clash API is already running and reachable, so this is mostly a window and a bundled static build. |
| Onboarding for an empty install | 📋 | A fresh install shows empty tables. A first-run path — add a subscription, or paste a link — would carry more than the current fallback copy. |

## Platform integration

| Feature | Status | Notes |
|---|---|---|
| System tray | 📋 | Connect/disconnect, active profile, and quick profile switching without opening the window. Close-to-tray instead of quit. The single most valuable desktop-only feature, and it has no NekoBox counterpart. |
| Launch at login | 🟡 | The `startup` setting is stored and shown, but nothing registers the app with the OS. Needs `tauri-plugin-autostart`. |
| Single-instance guard | 📋 | Two running copies would both drive the sing-box sidecar and fight over ports. |
| Deep links (`vless://`, `vmess://`, …) | 📋 | Desktop equivalent of NekoBox's intent-based import: register the URI schemes and import the profile on click. |
| Linux accent colour from the desktop portal | ❌ | Considered and dropped (2026-09-03). Kagerou uses its own theme colours on every platform. |

## Distribution & quality

| Feature | Status | Notes |
|---|---|---|
| Unit test coverage | ✅ | 163 Rust tests, 32 frontend tests; storage, parsers, config generation, supervisor, Clash API, and privilege planning are covered. |
| Continuous integration | 📋 | No `.github/` yet. Needs `cargo fmt --check` / `clippy` / `test` plus `pnpm lint` / `test` / `build` on pull requests. |
| Release builds | 📋 | No published binaries. Cross-platform bundles via `tauri-action`, triggered on tags. |
| Code signing and notarisation | 📋 | Unsigned builds mean a Gatekeeper warning on macOS and a SmartScreen warning on Windows. Needs certificates and secrets before it's worth automating. |
| In-app updates | 📋 | `tauri-plugin-updater` against the release feed. Depends on signed releases. |
| End-to-end connection verification | 📋 | `connect()` has never been run against a live server — only the generated config has been validated with `sing-box check`. Until someone does this, TUN mode and system proxy are unproven. |
| Version number | 📋 | Still `0.0.0` in `tauri.conf.json` and `app-meta.ts`. Bump when the first release is cut. |

---

## Parity with NekoBox for Android

NekoBox's feature surface, taken from its preference screens and menus, mapped
onto Kagerou. Sections above hold the detail; this table is for answering "does
Kagerou do X yet".

| NekoBox feature | Kagerou |
|---|---|
| Shadowsocks, VMess, VLESS, Trojan, Hysteria2, TUIC | ✅ |
| SOCKS, HTTP(S), SSH, WireGuard, ShadowTLS, AnyTLS | 📋 [Profiles](#profiles--subscriptions) |
| Trojan-Go, Mieru, NaiveProxy, Hysteria 1 | ❌ Not supported by the sing-box core Kagerou drives; NekoBox ships them as separate Android plugin APKs. |
| Shadowsocks plugins (simple-obfs, v2ray-plugin) | ❌ Android plugin APKs; no desktop equivalent. |
| Subscription import (Shadowsocks / Clash / v2rayN / sing-box formats) | ✅ |
| Subscription auto-update on an interval | 🟡 Setting exists, no scheduler. |
| Subscription User-Agent, deduplication, force-resolve | 📋 |
| Manual profile creation with per-protocol settings | 📋 Link paste only today. |
| QR code scan / share, clipboard and file import | 📋 Partly — pasted links work, QR and file import don't. |
| NFC sharing | ❌ Android hardware feature. |
| Backup and restore (groups, rules, settings) | 📋 |
| Profile groups, group ordering, group-wide URL test | 🟡 Groups and sorting done; group-wide test missing. |
| Proxy chains, front/landing proxy | 📋 |
| Custom config profiles and global config override | 📋 |
| Routing rules by domain, IP, port, source, network, protocol | 🟡 Domain and IP only. |
| Route by app / package | ❌ As-is: Android's per-app VpnService has no desktop counterpart. Desktop equivalent tracked as per-process routing 📋. |
| geoip / geosite assets with update management | 📋 |
| Bypass LAN | 🟡 Preset exists but is never applied. |
| DNS: remote/direct servers, domain strategy, DNS routing, FakeDNS | 📋 None of it. |
| Traffic sniffing, resolve destination | 📋 |
| TUN implementation choice, MTU, IPv6 mode | 📋 Hardcoded. |
| Mixed port, append HTTP proxy, allow LAN access | 🟡 Mixed inbound runs, but on a hardcoded loopback port. |
| Clash API + bundled web dashboard (Yacd) | 🟡 API is used internally; no dashboard is exposed. |
| Connection list with per-connection actions | 📋 |
| Per-profile traffic statistics | 📋 |
| Speed display and traffic notification | ✅ On the dashboard. Android notification behaviour (`speedInterval`, `showDirectSpeed`, `showGroupInNotification`) ❌ — desktop equivalent is the tray 📋. |
| Log viewer with configurable level | 🟡 Viewer done, level hardcoded. |
| Auto-connect on start | 📋 |
| Reset connections on network change / device wake | 📋 |
| Themes, dark mode | ✅ |
| Wake lock, metered network handling, Quick Settings tile, app shortcuts | ❌ Android platform features with no desktop counterpart. |
| System tray, launch at login, deep links | — Not in NekoBox; desktop-only, tracked in [Platform integration](#platform-integration). |
