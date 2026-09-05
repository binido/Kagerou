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
| 💡 | Idea — worth considering, not committed to. Talk it up or argue it down before anyone builds it. |

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
| TUN mode | ✅ | Confirmed end-to-end on macOS: connected to a real server with traffic routed through the TUN interface. The Windows (UAC) and Linux (`CAP_NET_ADMIN`, falling back to `pkexec`) elevation paths are implemented and unit-tested but have not been run on those systems — see [End-to-end verification](#distribution--quality). The mode is a stored setting read by `connect()`, so a change takes effect on the next connection rather than the current one. |
| System proxy | 🟡 | The settings row is present but disabled, and says so. Nothing sets the OS proxy: no `networksetup` (macOS), registry write (Windows), or GSettings/environment handling (Linux). The preference persists; only the effect is missing. |
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
| Live traffic telemetry | ✅ | Download and upload speed plus session totals, read from sing-box's own traffic and connections endpoints so they survive a frontend reload. Deliberately just the numbers — there is no traffic chart and none is planned. |
| Log viewer | ✅ | Streams the core's output live, with level detection. |
| Connection list | 📋 | The Clash API already reports every live connection (host, rule, upload, download, duration); nothing displays them. Include "close connection" and "close all". |
| Embedded sing-box dashboard | 📋 | NekoBox bundles Yacd. The Clash API is already running and reachable, so this is mostly a window and a bundled static build. |
| App icon has no backplate | 📋 | `app/public/logo-1024.png`, which every icon is generated from, is RGBA with a transparent background, so the mark floats on whatever sits behind it — in the dock and in Finder it reads as unfinished. Needs an opaque plate behind the mark (macOS also wants the padding from Apple's icon grid rather than a full-bleed square), then a regenerate with `tauri icon`. **Good first issue.** |
| Onboarding for an empty install | 📋 | A fresh install shows empty tables. A first-run path — add a subscription, or paste a link — would carry more than the current fallback copy. |

## Platform integration

| Feature | Status | Notes |
|---|---|---|
| System tray | 📋 | Connect/disconnect, active profile, and quick profile switching without opening the window. Close-to-tray instead of quit. The single most valuable desktop-only feature, and it has no NekoBox counterpart. |
| Launch at login | 🟡 | The `startup` setting is stored and shown, but nothing registers the app with the OS. Needs `tauri-plugin-autostart`. |
| Single-instance guard | 📋 | Two running copies would both drive the sing-box sidecar and fight over ports. |
| Deep links (`vless://`, `vmess://`, …) | 📋 | Desktop equivalent of NekoBox's intent-based import: register the URI schemes and import the profile on click. |
| Encrypted storage for profile keys | 💡 | Profile keys sit in plain text in the SQLite database and in the generated sing-box config, both under the OS app-data directory. That is normal for this class of app and the files are readable only by the account that owns them, so the threat model is worth arguing about before anyone writes code — a shared or backed-up machine is the case that would justify it. If it is worth doing, the OS keychain is a better home for the keys than a password on the database. |
| Linux accent colour from the desktop portal | ❌ | Considered and dropped (2026-09-03). Kagerou uses its own theme colours on every platform. |

## Distribution & quality

| Feature | Status | Notes |
|---|---|---|
| Unit test coverage | ✅ | 163 Rust tests, 32 frontend tests; storage, parsers, config generation, supervisor, Clash API, and privilege planning are covered. |
| Continuous integration | ✅ | `.github/workflows/ci.yml` runs `cargo fmt --check` / `clippy -D warnings` / `test` (including the ignored smoke test against the real core) and `pnpm lint` / `test` / `build` on every pull request and push to `main`. |
| Release builds | 🟡 | `.github/workflows/release.yml` builds macOS (both architectures), Linux and Windows bundles from a `v*` tag and attaches them to a draft release, after checking the tag against `tauri.conf.json`. Proven on `v0.1.0`: dmg, deb, rpm, AppImage, msi and exe all built. Nothing is downloadable until that draft is published. Linux ships x86_64 only. |
| Code signing and notarisation | 🟡 | Release builds are ad-hoc signed, which makes the macOS bundle structurally valid — without it the signature seals no resources and macOS reports the app as damaged rather than merely unverified. They are still not notarised, so a downloaded copy needs its quarantine flag cleared, and Windows still shows SmartScreen. Real signing needs a paid Apple certificate and a Windows one. |
| Update notification | ✅ | On launch the app asks GitHub for the latest release and, if it is newer than the running build, the sidebar links straight to it. Silent when the check fails or there are no releases. |
| In-app updates | 📋 | Downloading and applying the update in place, via `tauri-plugin-updater`. Depends on signed releases; today the notification just sends you to the release page. |
| End-to-end verification | 🟡 | `connect()` has been run against a real server on macOS in TUN mode. Windows and Linux have never been exercised beyond unit tests and `sing-box check`, so their elevation and TUN paths are unproven — first-hand reports from either are welcome. |
| Version number | ✅ | `tauri.conf.json` is the single source of truth — Vite injects it into the frontend, and neither package.json carries one. Currently `0.1.0`. |

---

## Known issues & internal debt

Not missing features — things that exist and work, but are built in a way
worth revisiting. Kept apart from the sections above so "we haven't built it"
never gets confused with "we built it badly".

| Issue | Status | Notes |
|---|---|---|
| `ProfileTable` renders every row twice | 📋 | The wide table and the narrow card list are both rendered on every pass, with CSS hiding whichever doesn't apply. Correct, but it doubles the DOM and the render work for every profile in every group. A `matchMedia` hook would render one or the other. |
| Dead profile-ordering plumbing | 📋 | `moveProfile` and `reorderProfiles` in the store, and the `move_profile` / `reorder_profiles` Tauri commands behind them, have no UI calling them. Either wire up manual reordering or delete all four; leaving them is a trap for the next person who greps for them. |
| Dialogs remount via their `key` | 📋 | `SourceDialog` and `ProfileGroupDialog` include the open flag in their React `key`, so every open and close throws the component away to reset its form state. It works, but resetting state on open would be the honest version. |

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
