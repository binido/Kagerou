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

Two markers cut across the statuses:

- **Good first issue** — self-contained, well-bounded, and doable without understanding the whole app. Start here.
- **Discuss first** — open an issue before writing code. Either the shape of the feature is undecided, or getting it wrong is expensive: this is a VPN client, and a routing or DNS mistake fails silently while looking like it works.

Anything unmarked sits in between: a clear task that touches more than one layer.

One section does not use this legend at all. [Accessibility & UX
audit](#accessibility--ux-audit) is a checklist rather than a status table,
because its rows are defects in shipped features rather than features — a
defect is outstanding or fixed, and there is no useful middle.

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
| System proxy | 🟡 | The settings row is present but disabled, and says so. Nothing sets the OS proxy: no `networksetup` (macOS), registry write (Windows), or GSettings/environment handling (Linux). The preference persists; only the effect is missing. **Discuss first.** |
| Inbound listen address and port | 📋 | The mixed inbound is hardcoded to `127.0.0.1:2080` and the Clash API to `127.0.0.1:9090`. Both should be settings, along with an "allow LAN access" toggle that binds `0.0.0.0`. **Good first issue.** |
| Configurable log level | ✅ | sing-box's log level is a stored setting in a Diagnostics section of the settings page; like TUN mode, it applies on the next connection. |
| Configurable connection-test URL | ✅ | The delay-test URL is a stored setting in Diagnostics, used by both the TCP and URL tests; blank values are rejected. |
| TUN tuning: MTU, stack, IPv6 mode | 📋 | Hardcoded to `stack: gvisor`, `172.19.0.1/30`, no MTU or IPv6 handling. gvisor rather than the kernel's own stack because the system one hung every TCP connection through the tunnel on CachyOS while UDP and DNS kept flowing; it costs throughput, since TCP is then reassembled in userspace, so making it a setting is worth more than it looks. The existing `tunInterface` setting is stored but never read by the config generator — either wire it up or drop it. **Discuss first.** |
| Auto-connect on launch | ✅ | Connect to the last active profile on startup, gated by a setting. The connect is spawned so the window is never delayed, and the snapshot carries the supervisor status so the UI learns about a pre-WebView connect. |
| Reset connections on network change / wake | 📋 | sing-box holds stale connections after a network switch or a laptop resume. NekoBox has both as toggles. **Discuss first.** |
| Custom config override | 📋 | Let a user append or override parts of the generated sing-box JSON, globally and per profile. Escape hatch for anything the UI doesn't expose. **Discuss first.** |

## Profiles & subscriptions

| Feature | Status | Notes |
|---|---|---|
| Protocols: VLESS, VMess, Trojan, Shadowsocks, Hysteria2, TUIC | ✅ | Parsed from links and generated into sing-box outbounds. |
| Subscription formats | ✅ | Plain or base64 URI lists, Clash-style YAML (`proxies:`), sing-box JSON (`outbounds`). |
| Profile groups | ✅ | Create, rename, move profiles between groups, reorder, drag-and-drop. |
| Subscription sources | ✅ | Add by URL or pasted key, manual refresh, remove. |
| Per-profile delay test | ✅ | One measurement: the latency of the whole path through the proxy, reported by sing-box's own API and shown in the Ping column. The TCP ping that used to sit beside it is gone — it measured the round trip to the proxy server rather than through it, which told a user nothing they could act on. |
| Group-wide delay test | ✅ | Each group's menu tests all its members concurrently (TCP or URL), then "clear results" resets both stored results and "delete unavailable" removes the profiles that failed the chosen method — never the active profile, never untested ones, behind a confirmation. |
| Export and sharing | 📋 | Copy a profile as a link, show it as a QR code, export a whole group to clipboard or file. |
| QR code import | 📋 | Scan a QR from an image file, the clipboard, or a screen region. |
| Backup and restore | 📋 | Export groups, profiles, routing rules, and settings as one JSON file, and import it back. NekoBox lets you pick which of the three to include. **Discuss first.** |
| Auto-select fastest (urltest group) | 📋 | Groups currently generate a plain selector. sing-box's `urltest` outbound gives automatic failover. **Discuss first.** |
| Proxy chains | 📋 | Route one proxy through another. NekoBox has both a chain profile type and per-group front/landing proxies. **Discuss first.** |
| Per-profile traffic statistics | 📋 | Bytes moved per profile, persisted, with a "clear statistics" action. |

## Routing & DNS

| Feature | Status | Notes |
|---|---|---|
| Routing rules | 🟡 | A rule is a single match string plus an outbound, classified into `domain` / `domain_suffix` / `ip_cidr` by shape. Rules can now be added and deleted from the page, and the match field says which of the three a pattern will become, because the classifier is total and answers `domain_suffix` just as readily for a wildcard or a pasted URL — both of which match nothing while sitting in the UI looking configured. New rules are appended, so ordering is still whatever order they were written in. NekoBox's rules also carry port, source, source port, network, and protocol; sing-box supports all of them. The storage schema needs to grow before the UI can. **Discuss first.** |
| Routing presets | 🟡 | `Bypass LAN` and `Block ads` are stored, toggle in the UI, and are then ignored — the config generator never reads them. Bypass LAN is a handful of private CIDRs and can be wired up on its own, though it is still routing and the CIDR list wants checking. Block ads cannot: it needs the `geosite` rule sets below, so it stays dark until those exist, and shipping the toggle meanwhile is the dishonest option. |
| DNS | ✅ | Queries go to a DoH resolver reached through the proxy, so the ISP sees an encrypted connection and not a list of every site visited — without a `dns` block sing-box fell back to the system resolver and leaked exactly that. Two things stay local by necessity: the proxy servers' own hostnames, which cannot be resolved through a tunnel that is not up (`route.default_domain_resolver`, mandatory since 1.14), and domains the routing rules send Direct, which would otherwise get a CDN address near the proxy rather than near the user. IPv4 only, matching the TUN interface, which is given no v6 address — an AAAA answer would route a connection into a tunnel that cannot carry it. Resolvers are not settings yet. |
| Configurable resolvers | 📋 | The remote resolver is Cloudflare over DoH and the local one is the system's, both hardcoded. NekoBox exposes both, plus a per-scope domain strategy. Wants a settings section of its own. **Good first issue.** |
| FakeDNS | 💡 | Hands the application an address out of `198.18.0.0/15` and resolves for real at connect time, which saves a round trip and makes domain routing exact — NekoBox turns it on by default. It also breaks anything expecting a real address: ping, some VoIP clients, address-based split tunnelling. Worth revisiting when there is something to diagnose it with. |
| Traffic sniffing | ✅ | A `sniff` rule runs before the routing rules, so a domain rule has a name to match on instead of the address the connection arrived as. Without it, `domain` and `domain_suffix` rules matched nothing while sitting in the UI looking configured. Written as a rule action, not an inbound field: those were removed in sing-box 1.13 and 1.14 rejects a config carrying them. |
| geoip / geosite rule sets | 📋 | Rules can't reference `geosite:category-ads` or `geoip:cn`. Needs rule-set support plus asset download and update, which NekoBox has as a separate "route assets" screen. **Discuss first.** |
| Per-process routing | 📋 | The desktop equivalent of NekoBox's per-app proxy: route by process name or path. sing-box supports `process_name` and `process_path` on Windows, macOS, and Linux. **Discuss first.** |
| Rule import / export | 📋 | Share rule sets as files, and ship a few sane defaults. |

## UI & UX

| Feature | Status | Notes |
|---|---|---|
| Dashboard, groups, sources, routing, logs, settings | ✅ | Six pages, all driven by the real backend. |
| Themes | 🟡 | Catppuccin and Kanagawa flavours. The four dark ones are clean; both light ones fail WCAG AA on every page, because the surface ramp is derived for a dark background and inverted when it is reused for a light one — see [the light theme row](#accessibility--ux-audit) in the audit. |
| Localisation | 🟡 | English and Russian. One hole: source refresh timestamps are stored as English prose by the backend and parsed back into translation keys by a regular expression, so anything the pattern misses reaches the screen untranslated — see [the timestamp row](#accessibility--ux-audit) in the audit. |
| Live traffic telemetry | ✅ | Download and upload speed, session totals, live connection count and session uptime, read from sing-box's own traffic and connections endpoints so they survive a frontend reload. A sparkline plots the last minute of speed, scaled to the window's own peak with a 1 Mbit/s floor and that peak labelled, and absorbs whatever height the rest of the dashboard leaves it. |
| Exit location lookup | ✅ | The dashboard names the exit country and city by asking a public service (ipwho.is), through the tunnel, what address it sees — replacing a guess made from the flag emoji in the profile's name, which most names do not carry. The exit IP sits beside it as the check that traffic really is leaving where it claims. A stored setting, on by default. With no lookup the line is simply absent — the flag it used to fall back to is already in the profile name above it — and the last session's location stays, unhighlighted, after disconnecting. Refreshed on connect, on a profile switch, and on demand, retrying a few times because the connection is announced while sing-box is still opening its inbound. |
| Log viewer | 🟡 | Streams the core's output live, with level detection. Three defects found by the audit: the timestamp column prints a raw ISO string, the INFO level uses a hardcoded hex that ignores the theme, and 500 rows render unvirtualised on every incoming line. |
| Connection list | 📋 | The Clash API already reports every live connection (host, rule, upload, download, duration); nothing displays them. Include "close connection" and "close all". |
| Embedded sing-box dashboard | 📋 | NekoBox bundles Yacd. The Clash API is already running and reachable, so this is mostly a window and a bundled static build. |
| App icon | ✅ | `assets/icon-source.svg` is the source: the mark on a Catppuccin Mocha plate, drawn on Apple's macOS grid (an 824×824 rounded square inset in a 1024 canvas) so it sits the same size as its neighbours in the dock. Regenerate the platform icons with `pnpm tauri icon assets/icon-source.png`. The mark loses its detail below about 48px, which would need separate small-size artwork inside the `.ico` and `.icns` — `tauri icon` scales a single source, so that is a manual job nobody has judged worth doing. |
| Theme-aware icon | ❌ | Considered and dropped (2026-09-05). Only a tray icon could genuinely follow the system theme — `Window::set_icon` is a no-op on macOS, and the bundle icon is baked in — so it would be a platform-specific detail hanging off a tray that does not exist yet, ahead of work that matters more. |
| Onboarding for an empty install | 📋 | A fresh install shows empty tables. A first-run path — add a subscription, or paste a link — would carry more than the current fallback copy. |

## Platform integration

| Feature | Status | Notes |
|---|---|---|
| Test without connecting | ✅ | Testing brings up a core of its own and shuts it down 30 seconds after the last test. It is never the connection's core, even when one is running: aiming a test means pointing a selector, and doing that to a live tunnel would silently reroute the user's traffic. Its own supervisor, its own config, ports 2081 and 9091, TUN off, and nothing announced to the UI as connected. |
| Ping that measures what the user asked | ✅ | A test now points the test core's selector at one profile and sends the configured URL through that core's own inbound twice, reporting the round trip of the second — the same thing NekoBox's `speedtest.UrlTest` reports in RTT mode, with the handshakes paid by the warm-up and left out. A result means the tunnel carried a real request, where sing-box's Clash `/delay` only approximated that and got it wrong: one hysteria2 node failed it at 5, 10 and 20 second timeouts while answering in 53 ms through the tunnel, five times running, its three siblings differing only by IP passing throughout. |
| Progress for a group test | ✅ | The run lives in the backend and walks the group in order — it has to, since aiming a test means holding the test core's selector — emitting each result as it lands. The page shows a bar with the count and a stop button, and results appear in their rows as they arrive. |
| System tray | ✅ | Connect and disconnect, switch to a recently used server, show the window, quit. Closing the window hides it here instead of dropping the tunnel — that is what the tray is for. The icon says whether the tunnel is up: a template image on macOS, which the system tints for the menu bar it is in, and the coloured mark elsewhere. The menu offers recent servers rather than all of them, since a subscription runs to hundreds and nobody picks from a menu that long. Its items go through the frontend's own actions rather than a second path into the backend. |
| Launch at login | ✅ | The `startup` setting stays the single source of truth: `update_settings` and a reconcile at every startup make the OS registration agree with it, via `tauri-plugin-autostart` (LaunchAgent on macOS). Registration is skipped under `tauri dev` so a debug binary never lands in login items. |
| Single-instance guard | ✅ | `tauri-plugin-single-instance` is registered before every other plugin; a second copy focuses the existing window instead of starting, so two supervisors never fight over the sidecar's ports. |
| Deep links (`vless://`, `vmess://`, …) | 📋 | Desktop equivalent of NekoBox's intent-based import: register the URI schemes and import the profile on click. **Discuss first.** |
| Encrypted storage for profile keys | 💡 | Profile keys sit in plain text in the SQLite database and in the generated sing-box config, both under the OS app-data directory. That is normal for this class of app and the files are readable only by the account that owns them, so the threat model is worth arguing about before anyone writes code — a shared or backed-up machine is the case that would justify it. If it is worth doing, the OS keychain is a better home for the keys than a password on the database. |
| Linux accent colour from the desktop portal | ❌ | Considered and dropped (2026-09-03). Kagerou uses its own theme colours on every platform. |

## Distribution & quality

| Feature | Status | Notes |
|---|---|---|
| Unit test coverage | ✅ | 163 Rust tests, 32 frontend tests; storage, parsers, config generation, supervisor, Clash API, and privilege planning are covered. |
| Continuous integration | ✅ | `.github/workflows/ci.yml` runs `cargo fmt --check` / `clippy -D warnings` / `test` (including the ignored smoke test against the real core) and `pnpm lint` / `test` / `build` on every pull request and push to `main`. |
| Release builds | 🟡 | `.github/workflows/release.yml` builds macOS (both architectures), Linux and Windows bundles from a `v*` tag and attaches them to a draft release, after checking the tag against `tauri.conf.json`. Proven on `v0.1.0`: dmg, deb, rpm, AppImage, msi and exe all built. Nothing is downloadable until that draft is published. Linux ships x86_64 only. |
| Code signing and notarisation | 🟡 | Release builds are ad-hoc signed, which makes the macOS bundle structurally valid — without it the signature seals no resources and macOS reports the app as damaged rather than merely unverified. They are still not notarised, so a downloaded copy needs its quarantine flag cleared, and Windows still shows SmartScreen. Real signing needs a paid Apple certificate and a Windows one. **Discuss first.** |
| Update notification | 🟡 | On launch the app asks GitHub for the latest release and, if it is newer than the running build, the sidebar links straight to it. Silent when the check fails or there are no releases. The link itself is `target="_blank"` with neither the `opener` plugin nor a `shell:allow-open` permission behind it, so in a real bundle it opens nothing — the notification arrives and then goes nowhere. See [the external links row](#accessibility--ux-audit) in the audit. |
| AUR package (`kagerou-bin`) | 📋 | Blocked (2026-09-05): AUR registration is not working, and the account plus its SSH key is the one part nobody else can do. Groundwork is known: build the package from the released `.deb`, which carries `usr/bin/kagerou`, `usr/bin/sing-box`, the desktop entry and hicolor icons. **Delete the bundled sing-box and `depends=('sing-box')` instead** — `/usr/bin/sing-box` belongs to `extra/sing-box`, so shipping our own there is a file conflict pacman refuses, and `sidecar_path()` resolves the core as "executable's directory + sing-box", which lands on the packaged one with no code change. Other dependencies, read from the binary: `gtk3`, `webkit2gtk-4.1`, `hicolor-icon-theme`. x86_64 only. A workflow on `release: published` — not on the tag, since our releases start as drafts — can bump `pkgver`, recompute the checksum, regenerate `.SRCINFO` and push. |
| Linux repositories | 📋 | Today a Linux user downloads a loose `.deb`, `.rpm` or AppImage from the release page and never hears about an update again. Repositories fix that: an apt repository (Debian/Ubuntu, hostable from GitHub Pages), COPR (Fedora), OBS (openSUSE, and it can build for several distros at once). Worth doing in that order — each is independent, and each is a chunk of packaging work rather than app work. |
| Flatpak | 💡 | The one package that would reach nearly every distro, and integrated into GNOME Software and KDE Discover — but it needs its sandbox question answered before anyone commits. A TUN interface needs `/dev/net/tun` and `CAP_NET_ADMIN`, and a Flatpak cannot simply elevate to get them; VPN clients on Flathub tend to hand the privileged half to something on the host. Establish whether TUN can work at all under Flatpak before packaging anything. |
| In-app updates | 📋 | Downloading and applying the update in place, via `tauri-plugin-updater`. Depends on signed releases; today the notification just sends you to the release page. **Discuss first.** |
| End-to-end verification | 🟡 | `connect()` has been run against a real server on macOS in TUN mode. Windows and Linux have never been exercised beyond unit tests and `sing-box check`, so their elevation and TUN paths are unproven — first-hand reports from either are welcome. |
| Version number | ✅ | `tauri.conf.json` is the single source of truth — Vite injects it into the frontend, and neither package.json carries one. |

---

## Known issues & internal debt

Not missing features — things that exist and work, but are built in a way
worth revisiting. Kept apart from the sections above so "we haven't built it"
never gets confused with "we built it badly".

Defects found by the frontend audit live in [their own
section](#accessibility--ux-audit) below, with a fix written out for each.

| Issue | Status | Notes |
|---|---|---|
| `ProfileTable` renders every row twice | 📋 | The wide table and the narrow card list are both rendered on every pass, with CSS hiding whichever doesn't apply. Correct, but it doubles the DOM and the render work for every profile in every group. A `matchMedia` hook would render one or the other. **Good first issue.** |
| Linux desktop entry and icon are malformed | 📋 | The generated `.desktop` has an empty `Categories=`, so the app lands uncategorised in application menus — fixed upstream with `bundle.category` in `tauri.conf.json`. The icon installed at `hicolor/256x256@2/apps/` is a 256×256 image: `@2` is not a hicolor directory, and the size is wrong for the name it was given. Both ship in the `.deb` and `.rpm` today. **Good first issue.** |
| Dead profile-ordering plumbing | 📋 | `moveProfile` and `reorderProfiles` in the store, and the `move_profile` / `reorder_profiles` Tauri commands behind them, have no UI calling them. Either wire up manual reordering or delete all four; leaving them is a trap for the next person who greps for them. |
| Dialogs remount via their `key` | 📋 | `SourceDialog` and `ProfileGroupDialog` include the open flag in their React `key`, so every open and close throws the component away to reset its form state. It works, but resetting state on open would be the honest version. **Good first issue.** |

---

## Accessibility & UX audit

A frontend audit run on 2026-09-08 against the Vercel Web Interface Guidelines
and WCAG 2.2 AA. Every finding below was reproduced in a real browser — the
frontend served by Vite with the Tauri IPC stubbed out, driven through Chrome
DevTools Protocol — rather than read off the source, so each row names the
symptom that was observed and not the rule that was broken.

This section uses checkboxes instead of the status column the rest of the file
uses. The rows above describe features, which are done or not done; the rows
here describe defects in features that already ship, and a defect is either
outstanding or fixed. The same honesty rule applies: a pull request that fixes
one of these ticks its box in the same commit, and a box is ticked only when
the verification step under it passes — not when the code looks right.

Four rows above were downgraded from ✅ to 🟡 by this audit: **Themes**,
**Localisation**, **Log viewer** and **Update notification**. Each names the
audit item that put it there, and goes back to ✅ when that item is ticked.

### Reproducing the measurements

The contrast numbers quoted below come from this snippet, pasted into the
DevTools console of a running `pnpm --dir app dev` with the theme under test
selected. It walks every element that owns a text node, resolves the nearest
opaque background, blends the foreground alpha into it, and reports anything
under the AA threshold for its size. An empty `fails` array is the pass
condition:

```js
const srgb = (c) => (c /= 255) <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4
const lum = (g) => 0.2126 * srgb(g[0]) + 0.7152 * srgb(g[1]) + 0.0722 * srgb(g[2])
const parse = (s) => (s.match(/[\d.]+/g) ?? []).map(Number)
const blend = (f, b) => f.length < 4 ? f : f.map((v, i) => i < 3 ? v * f[3] + b[i] * (1 - f[3]) : 1)
const bgOf = (el) => {
  for (let n = el; n && n !== document.documentElement; n = n.parentElement) {
    const c = parse(getComputedStyle(n).backgroundColor)
    if (c.length && (c[3] ?? 1) > 0.95) return c
  }
  return parse(getComputedStyle(document.documentElement).backgroundColor)
}
const ratio = (a, b) => (Math.max(lum(a), lum(b)) + 0.05) / (Math.min(lum(a), lum(b)) + 0.05)
const fails = []
for (const el of document.querySelectorAll('*')) {
  const text = [...el.childNodes].filter((n) => n.nodeType === 3).map((n) => n.textContent.trim()).join(' ').trim()
  if (!text) continue
  const cs = getComputedStyle(el)
  if (cs.visibility === 'hidden' || cs.display === 'none' || Number(cs.opacity) === 0) continue
  if (!el.getBoundingClientRect().width) continue
  const bg = bgOf(el)
  const r = ratio(blend(parse(cs.color), bg), bg)
  const size = parseFloat(cs.fontSize)
  const need = size >= 24 || (size >= 18.66 && Number(cs.fontWeight) >= 700) ? 3 : 4.5
  if (r < need) fails.push({ text: text.slice(0, 40), color: cs.color, size: cs.fontSize, ratio: +r.toFixed(2), need })
}
console.table(fails)
```

Run it on all six routes, in all eight themes. The keyboard checks are manual:
Tab from the top of each page to the bottom, then Escape out of every dialog,
dropdown and popover.

### Critical

Defects that make the app unusable for someone, or that hide a failure the
user needs to know about.

- [ ] **Reported modal and popover dismissal failure: not reproduced.**
  `app/src/components/ui/dialog.tsx`, `app/src/components/ui/popover.tsx`,
  `app/src/components/ui/alert-dialog.tsx`.

  The original audit reported four failures using debugging-protocol key
  events and synthetic KeyboardEvents: Escape reached document with
  defaultPrevented set, but data-state remained open. An overlay click at
  (150, 440) and Theme Picker dismissal reportedly failed too.

  A follow-up on 2026-09-12 at `f7fdcd7`, with the same
  `radix-ui@1.6.7` and `react@19.2.8`, passed 39 browser checks in each of
  three runs: Chromium with StrictMode, Chromium without StrictMode, and
  WebKit with StrictMode. The temporary StrictMode change was reverted.

  Each run opened add group, rename group, add key, rename profile, delete
  profile, add source, edit source, remove source, remove unavailable,
  edit rule, add rule, delete rule, and Theme Picker through their UI controls.
  Each was checked with Escape, an outside click, and Escape after reopening.
  Dialog and Popover closed on both inputs. AlertDialog closed on Escape
  and stayed open on outside clicks, as intended.

  These checks used the Vite development app with synthetic in-memory store
  data and browser keyboard/mouse events. They did not exercise a packaged
  Tauri app or real backend data. SourceDialog intentionally blocks dismissal
  while submitting; the checks above covered its idle state.

  The cause of the original observation remains unknown. StrictMode
  incompatibility was not confirmed, so no dependency update or dismissal
  override was applied. Keep this item open until the original failing
  environment and sequence can be reproduced. **Discuss first.**

- [ ] **The light themes fail WCAG AA across every page.**
  `app/src/themes/catppuccin.ts:12-23`, `app/src/themes/kanagawa.ts:150-165`.

  The surface ramp is inverted for light flavours. In Catppuccin,
  `surface0/1/2` sit *darker* than `base`, which reads as elevation on a dark
  background and as mud on a light one: in Latte every card becomes a grey
  slab and the text on it loses most of its contrast. Measured on `/groups`
  in Latte — 22 failures on that page alone, 22 more on `/sources`, 20 on
  `/routing-rules`:

  | Text | Colour | Measured | Needs |
  |---|---|---|---|
  | `Tokyo Premium Gateway 01` — primary text | `#4c4f69` | 3.69 | 4.5 |
  | `Managed by subscription` | `#6c6f85` | 2.28 | 4.5 |
  | `Imported` badge | `#40a02b` | 1.55 | 4.5 |
  | `180 ms` warning result | `#df8e1d` | 1.70 | 4.5 |
  | `Local` badge | `#7287fd` | 2.06 | 4.5 |
  | Preset description copy | `#9ca0b0` | 1.69 | 4.5 |

  Kanagawa Lotus is hand-mapped rather than derived, so it fares better, but
  `lotusGray2` (`#716e61`) still lands at 3.15–4.26 in every secondary label,
  and the semantic colours fail the same way.

  Invert the ramp for light flavours instead of patching call sites:

  ```ts
  // app/src/themes/catppuccin.ts:12
  const toTokens = (flavor: CatppuccinFlavor): ThemeTokens => {
    const { colors } = flavor
    const dark = flavor.dark

    return {
      // In a light flavour "higher" has to mean lighter, not darker.
      canvas: dark ? colors.base.hex : colors.mantle.hex,
      sidebar: dark ? colors.mantle.hex : colors.crust.hex,
      surface: dark ? colors.surface0.hex : colors.base.hex,
      surfaceElevated: dark ? colors.surface1.hex : colors.base.hex,
      surfaceSelected: dark ? colors.surface2.hex : colors.surface0.hex,
      surfaceHover: dark ? colors.surface1.hex : colors.mantle.hex,
      viewport: dark ? colors.crust.hex : colors.base.hex,
      ...
      textMuted: dark ? colors.subtext0.hex : colors.subtext1.hex,
      textQuiet: dark ? colors.overlay0.hex : colors.subtext0.hex,
    }
  }
  ```

  For Lotus, `textMuted: kanagawaPalette.lotusInk2` and
  `textQuiet: kanagawaPalette.lotusGray2`.

  The semantic colours need separate handling: Catppuccin's light flavours
  publish the same saturated `green`/`yellow`/`red` as the dark ones, and
  those cannot reach 4.5:1 on a near-white surface at any weight. Darken them
  for light flavours — `color-mix(in srgb, ${colors.green.hex} 65%, ${colors.crust.hex})`
  keeps the hue and buys the contrast — or hardcode a light-flavour triple.
  `ResultBadge` is the visible consumer, and a ping result that cannot be
  read is the one number on that page anybody looks at.

  Verify: the console snippet above returns an empty `fails` array on all six
  routes in Latte and Lotus, and still does in the four dark themes.
  **Discuss first** — this changes how every light theme looks, and the call
  between "darken the accents" and "swap the ramp only" is a design decision.

- [ ] **A backend error leaves a permanently blank window.**
  `app/src/App.tsx:20-25`, `app/src/store/kagerou-store.ts:134-141`.

  `hydrate()` awaits `get_app_state` with no `catch`, and `App` returns
  `null` until `hydrated` flips. Any failure on that path — a corrupt
  database, a migration that throws, a command that panics — and the user
  gets a window with nothing in it and no text explaining why. The
  unhandled rejection lands in a console they will never open. This is
  exactly what happens today when the frontend is served outside Tauri,
  which is how the audit found it.

  ```ts
  // app/src/store/kagerou-store.ts:134
  hydrate: async () => {
    subscribeToBackendEvents()
    try {
      const snapshot = await kagerouApi.getAppState()
      set({ ...applySnapshot(snapshot), hydrated: true })
    } catch (error) {
      set({ hydrated: true, hydrateError: backendErrorMessage(error, 'Failed to load app state') })
      return
    }
    // Deliberately not awaited: a slow or unreachable GitHub must not hold
    // up the first paint, and the command never rejects.
    void kagerouApi.checkForUpdate().then((updateAvailable) => set({ updateAvailable }))
  },
  ```

  ```tsx
  // app/src/App.tsx:25
  if (!hydrated) return null
  if (hydrateError) {
    return <p className="p-8 text-[13px] text-bad" role="alert">{hydrateError}</p>
  }
  ```

  `hydrateError: string | null` goes into `KagerouStore` in
  `app/src/types/kagerou.ts` alongside `hydrated`. The message wants a real
  next step in it, not just the failure — where the database lives, and that
  deleting it starts fresh — since a corrupt store is the likeliest cause
  and the user cannot act on "Failed to load app state" alone.

  Verify: point the app at an unreadable app-data directory, or throw from
  `get_app_state`, and confirm the window says something. **Good first issue.**

- [x] **Fourteen actions fail silently.**
  `app/src/store/kagerou-store.ts:151, 159, 191, 221, 263, 276, 285, 294, 304, 348, 355, 362, 370, 375`.

  Every mutating action catches its error into `console.error` and stops
  there. The worst case is `toggleConnection`: press Connect, have
  `connect()` reject because the sidecar is missing or the TUN elevation
  prompt was denied, and nothing happens at all — the dial does not move,
  no message appears, and the app looks like it ignored the click. The
  optimistic ones are quietly worse: `updateSettings` and `setPreset` paint
  the new state, fail to persist it, and leave the UI disagreeing with the
  database until the next reload.

  `sonner` is already wired up and already used this way in
  `app/src/pages/SourcesPage.tsx:77`, so this is reporting, not plumbing:

  ```ts
  // app/src/store/kagerou-store.ts:145
  toggleConnection: async () => {
    const { connected } = get()
    try {
      if (connected) await kagerouApi.disconnect()
      else await kagerouApi.connect()
    } catch (error) {
      toast.error(backendErrorMessage(error, i18n.t('common:feedback.connectFailed')))
    }
  },
  ```

  The store is outside React, so it needs `i18n` imported directly from
  `@/i18n` rather than a `useTranslation` hook, and the new copy needs rows
  in both `app/src/locales/en/common.json` and `.../ru/common.json`.

  Split the fourteen by what the user loses. `toggleConnection`,
  `selectProfile`, `deleteProfile`, `startGroupTest`, `cancelGroupTest` and
  `deleteUnavailableProfiles` are direct actions and must report. The
  optimistic ones — `setProfileGroupOpen`, `setPreset`, `selectRule`,
  `updateRule`, `setTheme`, `updateSettings` — must report *and* roll back,
  or they will keep lying about what is stored.

  Verify: stop the sing-box sidecar from resolving and press Connect; a
  toast appears naming the reason.

  Fixed as proposed: every mutating action reports through
  `backendErrorMessage`, so the backend's own message reaches the toast and
  the `common:feedback.*` copy is only the fallback. The six optimistic ones
  roll back from a fresh `get_app_state` snapshot — the same re-sync
  `selectProfile` already did on failure — instead of captured values, so
  the UI cannot disagree with what the database actually accepted. Two
  things a snapshot does not restore get explicit handling: `setTheme`
  reverts the persisted theme id, and `updateRule` reverts the
  `rulesChangedSinceConnect` flag. `refreshExitLocation` keeps its silent
  failure — it is deliberate and documented — and the actions that return
  null/false into dialogs were never part of this row, since the pages show
  those errors inline. Failure paths and rollbacks are covered by store
  tests.

- [ ] **No skip link, and `<main>` has no accessible name.**
  `app/src/components/layout/AppShell.tsx:9`.

  Six sidebar links sit ahead of the content on every page, and a keyboard
  or screen-reader user walks all six on every navigation.

  ```tsx
  // app/src/components/layout/AppShell.tsx:6
  <div className="flex min-h-screen w-full overflow-x-clip bg-canvas text-primary">
    <a
      className="sr-only focus:not-sr-only focus:absolute focus:left-3 focus:top-3 focus:z-50 focus:rounded-md focus:bg-raised focus:px-3 focus:py-2 focus:focus-ring"
      href="#main"
    >
      {t('a11y.skipToContent')}
    </a>
    <Sidebar />
    <main aria-label={t('a11y.mainContent')} className="min-w-0 flex-1" id="main" tabIndex={-1}>
      <Outlet />
    </main>
  </div>
  ```

  `tabIndex={-1}` is what makes the jump actually move focus rather than
  only the scroll position. `AppShell` currently takes no translation hook,
  so it needs `useTranslation('common')` and two new keys in both locales.

  Verify: Tab once from a fresh page load, confirm the link appears, press
  Enter, then Tab again and land inside the page content rather than back
  in the sidebar. **Good first issue.**

- [x] **`aria-selected` on a `<tr>` inside a plain table.**
  `app/src/components/routing/RoutingRulesTable.tsx`.

  `aria-selected` is only defined for the `row` role inside `grid` or
  `treegrid`, so in a `table` it was dropped and the selected rule was
  announced identically to every other row. Two more problems sat in the same
  element: `event.preventDefault()` on Space killed page scrolling while a row
  had focus, and the row was a focusable element containing another focusable
  element.

  Fixed the way this entry proposed, by dropping the row interaction rather
  than promising grid keyboard behaviour the table does not implement. The
  match cell now holds a real button carrying `aria-current` on the selected
  rule, which needs no roving tabindex and reads as "current" rather than as
  nothing. The delete button added alongside edit would otherwise have made
  the nesting worse, not better.

- [ ] **Fonts are fetched from Google at every launch.**
  `app/src/index.css:1`.

  ```css
  @import url('https://fonts.googleapis.com/css2?family=IBM+Plex+Mono...');
  ```

  A VPN client is offline by definition until it connects, so the first
  paint of the first launch is the case where this fails. The layout is
  built on fixed pixel sizes (`text-[13px]`, `h-10`, `w-[104px]`), so a
  fallback to the system sans-serif does not degrade gracefully — it
  reflows. There is a second reason to care in this app specifically: a
  privacy tool that phones a third party on startup, before any tunnel
  exists, is making a request the user did not agree to.

  ```bash
  pnpm --dir app add @fontsource/inter @fontsource/inter-tight @fontsource/ibm-plex-mono
  ```

  ```css
  /* app/src/index.css:1 */
  @import "@fontsource/inter/400.css";
  @import "@fontsource/inter/500.css";
  @import "@fontsource/inter/600.css";
  @import "@fontsource/inter-tight/500.css";
  @import "@fontsource/inter-tight/600.css";
  @import "@fontsource/inter-tight/700.css";
  @import "@fontsource/ibm-plex-mono/400.css";
  @import "@fontsource/ibm-plex-mono/500.css";
  ```

  Three new dependencies is more than this file usually welcomes, but they
  are build-time asset packages with no runtime, and the weights listed are
  exactly the ones `index.css:73-75` declares. Subsetting to `latin` and
  `cyrillic` matters here — the app ships Russian.

  Verify: launch with networking disabled and confirm the type looks
  identical to a launch with networking on. **Good first issue.**

### Improvements

Real defects, none of them blocking. Roughly in the order they are worth
doing.

- [ ] **The focus ring is drawn at 50% alpha.**
  `app/src/index.css:81-83`.

  ```css
  * { @apply border-border outline-ring/50; }
  ```

  This sets `outline-color` on everything and wins over the `focus-ring`
  utility's own colour, so every focus indicator in the app renders as
  half-transparent lavender: measured at roughly 1.9:1 against the sidebar,
  where WCAG 2.4.11 wants 3:1. On the active navigation item, which already
  has a `bg-selected` background, it is close to invisible — the audit's
  first tab-through read as "no focus ring at all" until the computed
  styles were dumped.

  ```css
  /* app/src/index.css:81 */
  * {
    @apply border-border;
  }
  ```

  Nothing depends on the global outline colour: every interactive component
  declares its own `focus-visible:` treatment, and the `focus-ring` utility
  at `index.css:141` already specifies a full-opacity `var(--lavender)`.

  Verify: Tab through the sidebar and see the ring on every item, including
  the active one, in all eight themes. **Good first issue.**

- [ ] **Log rows print a raw ISO timestamp.**
  `app/src/components/logs/LogRow.tsx:28`, `app/src/store/kagerou-store.ts:46`.

  The store stores `new Date().toISOString()` and the row renders it
  verbatim, so the 186px timestamp column reads
  `2026-09-08T17:52:46.522Z` on every line.

  ```ts
  // app/src/lib/formatters.ts
  const logTime = new Intl.DateTimeFormat(undefined, {
    hour: '2-digit', minute: '2-digit', second: '2-digit', hour12: false,
  })

  /** ISO timestamp from the store to the `HH:MM:SS` the log column shows. */
  export const formatLogTimestamp = (iso: string) => logTime.format(new Date(iso))
  ```

  Keep the ISO string in the entry — it is the sortable form, and a future
  "copy logs" wants it — and format only at the point of render. Watch the
  filter: `app/src/pages/LogsPage.tsx:16` searches
  `${entry.timestamp} ${entry.level} ${entry.message}`, so unless it
  searches the formatted string too, typing `17:52` finds nothing while the
  screen is full of it.

  The column can also narrow considerably once it holds eight characters
  instead of twenty-four — `app/src/index.css:159` has it at `186px`.
  **Good first issue.**

- [ ] **A hardcoded hex colour in the log level palette.**
  `app/src/components/logs/LogRow.tsx:5`.

  ```ts
  const levelClasses: Record<LogEntry['level'], string> = {
    INFO: 'text-[#b8b1cf]',
    ...
  ```

  A fixed lavender-grey, chosen for a dark background, rendered unchanged on
  Latte and Lotus. It is the only place in the frontend that bypasses the
  theme tokens. Replace with `text-body`; `WARN` and `ERROR` on the lines
  below already use `text-warn` and `text-bad` correctly.
  **Good first issue.**

- [ ] **Five hundred log rows render unvirtualised.**
  `app/src/store/kagerou-store.ts:18`, `app/src/components/logs/LogViewer.tsx:18`.

  `MAX_LOG_ENTRIES = 500` and the viewer maps all of them. Each row then
  runs `HighlightedMessage`, which compiles a fresh `RegExp` per row per
  render at `LogRow.tsx:13`. While connected, sing-box emits continuously
  and the whole list re-renders on every entry.

  The cheap fix needs no dependency and no windowing library:

  ```tsx
  // app/src/components/logs/LogViewer.tsx:17
  <div
    className="space-y-0 px-4 py-4"
    id="log-list"
    style={{ contentVisibility: 'auto', containIntrinsicSize: '0 21px' }}
  >
  ```

  `content-visibility: auto` skips layout and paint for off-screen rows
  while keeping them in the DOM, so Ctrl+F and text selection still work —
  which a windowing library would break. Hoist the regex out of the render
  in the same pass:

  ```tsx
  // app/src/components/logs/LogRow.tsx:10
  function HighlightedMessage({ message, query }: { message: string; query: string }) {
    const needle = query.trim()
    const pattern = useMemo(
      () => needle ? new RegExp(`(${needle.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')})`, 'ig') : null,
      [needle],
    )
    if (!pattern) return message
    ...
  ```

  Note the existing character class at `LogRow.tsx:13` is subtly wrong —
  `[.*+?^${}()|[\\]\\]` escapes the backslash twice and never closes on
  `]` — so a query containing `]` throws. Fixing it belongs here.

- [ ] **New log lines are never announced.**
  `app/src/components/logs/LogViewer.tsx:17`.

  The log viewer is the one place in the app that updates on its own, and
  it does so silently. `aria-live="polite"` with `aria-relevant="additions"`
  announces appended rows without re-reading the whole list:

  ```tsx
  <div aria-live="polite" aria-relevant="additions" className="space-y-0 px-4 py-4" id="log-list">
  ```

  Worth pairing with the level filter that does not exist yet — announcing
  every INFO line from a busy core is its own kind of unusable, so consider
  scoping the live region to WARN and ERROR. **Discuss first.**

- [ ] **Switch labels are not clickable.**
  `app/src/components/settings/SettingSwitchRow.tsx:15`,
  `app/src/components/routing/PresetSwitchRow.tsx:21`.

  The label is a `<span>` or `<p>` sitting beside the control with no
  association, so the hit target is the 32×18px switch rather than the full
  row. Screen-reader users are fine — the switch carries `aria-label` — but
  everyone else is aiming at a thumbnail.

  ```tsx
  // app/src/components/settings/SettingSwitchRow.tsx:11
  export function SettingSwitchRow({ id, label, description, checked, disabled = false, onChange }: SettingSwitchRowProps) {
    return (
      <div className="flex min-h-14 items-center justify-between gap-8 border-b border-hairline/55">
        <label className="min-w-0 cursor-pointer" htmlFor={id}>
          <span className="block text-[14px] leading-5 text-body">{label}</span>
          {description ? <p className="mt-1 text-[11px] leading-4 text-muted-copy">{description}</p> : null}
        </label>
        <Switch
          checked={checked}
          className="shrink-0 data-checked:bg-lavender data-unchecked:bg-raised"
          disabled={disabled}
          id={id}
          onCheckedChange={onChange}
        />
      </div>
    )
  }
  ```

  Drop the `aria-label` when the `<label>` goes in, or the two compete and
  the visible text loses. `id` becomes a required prop, so every call site in
  `app/src/pages/SettingsPage.tsx` needs one. Same shape for
  `PresetSwitchRow`. **Good first issue.**

- [ ] **Preset labels are guessed from a hardcoded id.**
  `app/src/components/routing/PresetSwitchRow.tsx:9-11`.

  ```ts
  const presetCopyKeys = (id: string) => id === 'block-ads'
    ? { label: 'presets.blockAds.label', ... }
    : { label: 'presets.bypassLan.label', ... }
  ```

  Anything that is not `block-ads` renders as "Bypass LAN", including a
  preset that is neither. Confirmed live: two different presets drew
  identical rows. The backend already sends `label` and `description` on
  every preset, so the fallback should be those rather than a guess — keep
  the id lookup for the two known presets that want translating, and fall
  through to the backend copy for anything else.

  This one is a trap for whoever adds a third preset, and it will look like
  a backend bug when it lands. **Good first issue.**

- [ ] **The theme list is eight tab stops, and its arrow keys disagree with its grouping.**
  `app/src/components/settings/ThemeFlavorRow.tsx:36`,
  `app/src/components/settings/ThemePicker.tsx:55-73, 108`.

  Every flavour row is a native `<button>` carrying `role="radio"`, so each
  is in the tab order — a radio group should be a single stop with arrows
  moving inside it. Separately, `ThemePicker` declares one `radiogroup` per
  pack but `handleRowKeyDown` cycles through `allThemes`, so the arrows walk
  out of the group that was announced.

  ```tsx
  // app/src/components/settings/ThemeFlavorRow.tsx:27
  <Button ref={ref} aria-checked={active} role="radio" tabIndex={active ? 0 : -1} ... />
  ```

  Then pick one of the two shapes and make the keys match it: one
  `radiogroup` around the whole list, keeping the existing wrap-around
  arrows, or per-pack groups with arrows that stop at each pack's edges. The
  first is less work and matches what the arrows already do — move the
  `role="radiogroup"` from `ThemePicker.tsx:108` up to the wrapper at line
  101 and give it a single label.

- [ ] **Dialog validation errors are not tied to their field.**
  `app/src/components/profiles/ProfileGroupDialog.tsx:71-74`,
  `app/src/components/sources/SourceDialog.tsx:95`.

  The message renders as a sibling above the footer. The input gets no
  `aria-invalid`, its `aria-describedby` points only at the helper text, and
  focus stays wherever it was. `role="alert"` means the text is read once,
  but a user who tabs back to the field hears nothing about it being wrong.

  ```tsx
  // app/src/components/profiles/ProfileGroupDialog.tsx:71
  <Input
    aria-describedby={error ? 'profile-group-error profile-group-helper' : 'profile-group-helper'}
    aria-invalid={Boolean(error)}
    autoFocus
    id="profile-group-name"
    ref={inputRef}
    ...
  />
  ...
  {error ? <FieldError className="text-[11px]" id="profile-group-error">{error}</FieldError> : null}
  ```

  And return focus to the field on a rejected submit — `handleSubmit:45` and
  `:51` both `return` without moving it:

  ```tsx
  if (!trimmed) {
    setError(t('dialogs.group.empty'))
    inputRef.current?.focus()
    return
  }
  ```

  The error also belongs directly under the input rather than above the
  footer, so the eye finds it where the mistake is. **Good first issue.**

- [ ] **URL fields are typed as plain text and spell-checked.**
  `app/src/components/sources/SourceDialog.tsx:87, 92`,
  `app/src/components/settings/SettingTextRow.tsx:37`.

  The subscription URL, the pasted protocol key and the connection-test URL
  are all `type="text"` with spellcheck on, so the browser underlines
  base64 payloads and hostnames in red.

  ```tsx
  <Input
    autoComplete="off"
    inputMode="url"
    spellCheck={false}
    type={type === 'url' ? 'url' : 'text'}
    ...
  />
  ```

  Leave the key field as `type="text"` — `vless://` is not a URL the browser
  validator recognises — but it still wants `spellCheck={false}`.

  In the same files: `SettingTextRow.tsx:23` and `SettingNumberRow.tsx:26`
  call `onChange` on every keystroke, which walks through the store to a
  SQLite write per character — a 40-character test URL is 40 writes. Both
  already have an `onBlur` doing validation; move the persist there and keep
  the keystroke handler local. **Good first issue.**

- [ ] **`transition-all` on three primitives.**
  `app/src/components/ui/button.tsx:8`, `app/src/components/ui/badge.tsx:8`,
  `app/src/components/ui/switch.tsx:20`.

  Animates every animatable property including `width`, `height` and
  `background`, none of which are compositor-friendly. Buttons in this app
  change size — the profile select button swaps "Use" for "Selected" — so
  this is a real reflow, not a theoretical one. List the properties:

  ```
  transition-[color,background-color,border-color,box-shadow,transform]
  ```

  The rest of the `ui/` primitives already use
  `transition-[color,box-shadow]`, so this is bringing three files in line
  with the other ten. **Good first issue.**

- [ ] **Relative timestamps never advance, and English leaks through the parser.**
  `app/src/components/sources/SourceCard.tsx:35-58`,
  `src-tauri/src/commands.rs:826, 861, 924`,
  `src-tauri/src/storage/sources.rs:115`.

  The backend writes the literal string `"Updated just now"` into the
  database, and the frontend parses it back into an i18n key with a regular
  expression. Two consequences. First, the branches matching
  `/^Updated (\d+) min ago$/` and `days ago` are dead code — nothing ever
  writes those strings — so a source refreshed a week ago still reads
  "Updated just now" forever. Second, `SourceCard.tsx:58` falls through to
  the raw stored value when the pattern misses, which puts untranslated
  English on screen.

  Store a timestamp and format it at the edge:

  ```rust
  // src-tauri/src/commands.rs:924
  last_refresh: Some(&chrono::Utc::now().to_rfc3339()),
  ```

  ```ts
  // app/src/lib/formatters.ts
  const relative = (language: string) => new Intl.RelativeTimeFormat(language, { numeric: 'auto' })

  /** Stored RFC 3339 timestamp to "5 minutes ago" in the active language. */
  export const formatLastRefresh = (iso: string, language: string) => {
    const minutes = Math.round((Date.now() - Date.parse(iso)) / 60_000)
    if (minutes < 60) return relative(language).format(-minutes, 'minute')
    if (minutes < 1440) return relative(language).format(-Math.round(minutes / 60), 'hour')
    return relative(language).format(-Math.round(minutes / 1440), 'day')
  }
  ```

  Then delete `sourceTimestampKey` and the five `card.updated*` keys from
  both locale files. This needs a migration decision: existing rows hold
  prose, not timestamps, and `Date.parse` returns `NaN` for them. Either
  migrate them to `NULL` and render an em dash, or add a migration that
  stamps them with the migration's own time — the first is honest, the
  second is prettier. **Discuss first**, because it changes the storage
  schema's meaning and touches both sides at once.

- [ ] **A decorative arrow is read aloud.**
  `app/src/components/sources/SourceCard.tsx:106`.

  `<span className="text-lavender">→</span>` is announced as "right arrow"
  in the middle of a sentence. Add `aria-hidden="true"`.
  **Good first issue.**

- [ ] **A disabled switch looks almost enabled.**
  `app/src/components/settings/SettingSwitchRow.tsx:18`,
  `app/src/components/ui/switch.tsx:20`.

  `disabled` gives only `opacity-50`, which on the System proxy row reads as
  a slightly dimmer version of a working control rather than a disabled one.
  The explanatory copy beside it carries the whole message. Give the state a
  shape of its own:

  ```
  data-disabled:bg-transparent data-disabled:ring-1 data-disabled:ring-hairline
  ```

  Ties into the System proxy row in
  [Core & connectivity](#core--connectivity), which is 🟡 for the same
  reason. **Good first issue.**

- [ ] **Dead `aria-hidden` and an unnamed region on group panels.**
  `app/src/components/profiles/ProfileGroupCard.tsx:59-60`.

  ```tsx
  {group.open ? (
    <div aria-hidden={!group.open} id={`${group.id}-panel`} role="region">
  ```

  The node only exists when `group.open` is true, so `aria-hidden` is
  permanently `false` and can go. `role="region"` without an accessible name
  is skipped by screen readers entirely, which wastes the `aria-controls`
  wiring the header button already has:

  ```tsx
  <div aria-label={groupLabel} id={`${group.id}-panel`} role="region">
  ```

  **Good first issue.**

### Tauri-specific

Places where the frontend behaves like a web page inside what is meant to be
a desktop application.

- [ ] **External links cannot open.**
  `app/src/components/settings/SettingsFooter.tsx:16-25`,
  `app/src/components/layout/SidebarUpdateNotice.tsx:25`.

  Both use `<a href={...} target="_blank">`. Tauri v2 does not implement
  `window.open` by default, and
  `src-tauri/capabilities/default.json` grants only `core:default` and
  `autostart:default` — no `shell:allow-open`, and the `opener` plugin is
  not among the four in `src-tauri/Cargo.toml:19-22`. So the click does
  nothing, or navigates the app window itself to GitHub with no way back.
  The update notice is the only path a user has to a new release, which
  makes this the more serious of the two.

  ```bash
  pnpm --dir app add @tauri-apps/plugin-opener
  cd src-tauri && cargo add tauri-plugin-opener
  ```

  ```json
  // src-tauri/capabilities/default.json
  "permissions": ["core:default", "autostart:default", "opener:allow-open-url"]
  ```

  ```tsx
  // app/src/components/settings/SettingsFooter.tsx:16
  import { openUrl } from '@tauri-apps/plugin-opener'

  <button
    className="inline-flex min-w-0 items-center gap-1.5 text-muted-copy transition-colors hover:text-lavender-hi focus-visible:focus-ring"
    onClick={() => { void openUrl(KAGEROU_REPOSITORY_URL) }}
    type="button"
  >
  ```

  A `<button>` rather than an `<a>` is right here despite the usual rule:
  this is not navigation within the document, it is a request to the host
  OS, and an `<a>` that does nothing is worse than a button that works. Both
  places already carry `aria-label`, so the announcement does not change.
  Restrict the permission to the two known URLs if the capability schema
  allows it — `opener:allow-open-url` with a `urls` list is narrower than
  handing the frontend an arbitrary opener.

  Verify: in a built bundle, not `tauri dev`, click both links and watch the
  system browser open. **Good first issue.**

- [ ] **The interface selects like a web page.**
  `app/src/index.css:90-99`.

  `getComputedStyle(document.body).userSelect` returns `auto`, so dragging
  across the profile list highlights headings, counts and labels. Nothing
  else about the window says "browser", and this does.

  ```css
  /* app/src/index.css:90 */
  body {
    @apply min-w-0 bg-canvas text-foreground;
    min-height: 100vh;
    margin: 0;
    user-select: none;
    ...
  }
  ```

  Then hand selection back to the things people genuinely copy — the masked
  subscription value at `app/src/components/sources/SourceCard.tsx:99`, log
  message text at `app/src/components/logs/LogRow.tsx:30`, and every input:

  ```css
  /* app/src/index.css:157, @layer components */
  input,
  textarea,
  [data-selectable] {
    user-select: text;
  }
  ```

  Verify: drag across the groups page and select nothing; drag across a log
  line and select the message. **Good first issue.**

- [ ] **The WebView's own context menu is reachable.**

  No `contextmenu` handler anywhere in the frontend, so right-clicking opens
  the platform WebView's menu — reload, back, inspect, depending on the
  platform — none of which belongs in this app.

  ```tsx
  // app/src/main.tsx:8
  if (!import.meta.env.DEV) {
    document.addEventListener('contextmenu', (event) => event.preventDefault())
  }
  ```

  The `DEV` guard is not optional: without it, `tauri dev` loses its
  inspector. Leaving right-click dead is the minimum; a real context menu on
  a profile row would be better, and is worth a row of its own if anyone
  wants it. **Good first issue.**

**Window dragging — checked, nothing to do.** Not a checkbox, because there is
nothing outstanding; it is here so the next audit does not raise it again.
There are zero
`data-tauri-drag-region` attributes in the DOM, and
`src-tauri/tauri.conf.json:14-21` does not set `"decorations": false`, so the
window keeps its native title bar and drags by it. Adding the attribute now
would gain nothing.

It becomes required the moment anyone builds a frameless window, and the trap
is worth writing down in advance: the attribute swallows clicks across its
whole subtree, so it cannot go on a container that holds buttons. The working
shape is an absolutely positioned sibling behind the header content:

```tsx
<header className="relative flex items-start justify-between gap-6">
  <div aria-hidden="true" className="absolute inset-0" data-tauri-drag-region />
  <div className="relative">...</div>
</header>
```

### Checked and clean

Recorded so the next audit does not re-derive them, and so a regression here
is visible as a change rather than a discovery.

| Checked | Result |
|---|---|
| `-webkit-tap-highlight-color` | Set to transparent on buttons, `app/src/index.css:101-104`. |
| `color-scheme` and `<meta name="theme-color">` | Both follow the active theme, `app/src/themes/runtime.ts:70, 76`. |
| `prefers-reduced-motion` | Global duration clamp at `app/src/index.css:185`; `ThemePicker` additionally zeroes its own `motion` animations. |
| Horizontal overflow and stray scrollbars | None. `scrollWidth === clientWidth` at 1280×800 and at the 860×560 minimum from `tauri.conf.json`; no element overflows its container. |
| Minimum-window layout | Holds at 860×560. The profile table switches to its compact card list, the sidebar collapses to icons, long names truncate. |
| Draggable images | None to suppress; the only image is an inline SVG mark. |
| Icon-only buttons | All carry `aria-label`; decorative icons are `aria-hidden`. Spot-checked across all six pages. |
| Async status regions | The log search count (`LogToolbar.tsx:45`) and the ping readout (`ConnectionTrafficReadouts.tsx:65`) are `role="status"`. |

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
| Profile groups, group ordering, group-wide URL test | ✅ Groups, sorting, group-wide testing and the delete/clear actions are all in. |
| Proxy chains, front/landing proxy | 📋 |
| Custom config profiles and global config override | 📋 |
| Routing rules by domain, IP, port, source, network, protocol | 🟡 Domain and IP only. |
| Route by app / package | ❌ As-is: Android's per-app VpnService has no desktop counterpart. Desktop equivalent tracked as per-process routing 📋. |
| geoip / geosite assets with update management | 📋 |
| Bypass LAN | 🟡 Preset exists but is never applied. |
| DNS: remote/direct servers, domain strategy, DNS routing, FakeDNS | 🟡 Resolving works and does not leak, and DNS routing mirrors the Direct rules. Choosing the resolvers is not exposed yet; FakeDNS is deliberately off. |
| Traffic sniffing, resolve destination | 🟡 | Sniffing done. Resolving a sniffed domain back to an address, so `ip_cidr` rules can match it, still missing and tied to the DNS row. |
| TUN implementation choice, MTU, IPv6 mode | 📋 Hardcoded. |
| Mixed port, append HTTP proxy, allow LAN access | 🟡 Mixed inbound runs, but on a hardcoded loopback port. |
| Clash API + bundled web dashboard (Yacd) | 🟡 API is used internally; no dashboard is exposed. |
| Connection list with per-connection actions | 📋 |
| Per-profile traffic statistics | 📋 |
| Speed display and traffic notification | ✅ On the dashboard. Android notification behaviour (`speedInterval`, `showDirectSpeed`, `showGroupInNotification`) ❌ — desktop equivalent is the tray 📋. |
| Log viewer with configurable level | 🟡 Viewer done, level hardcoded. |
| Auto-connect on start | ✅ |
| Reset connections on network change / device wake | 📋 |
| Themes, dark mode | ✅ |
| Wake lock, metered network handling, Quick Settings tile, app shortcuts | ❌ Android platform features with no desktop counterpart. |
| System tray, launch at login, deep links | — Not in NekoBox; desktop-only, tracked in [Platform integration](#platform-integration). |
