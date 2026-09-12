# Kagerou

> A clean, cross-platform sing-box VPN client for the desktop.

Kagerou is a native desktop client for [sing-box](https://sing-box.sagernet.org/). It turns proxy subscriptions and profiles into a simple connection workflow with TUN mode, routing rules, live traffic data, and a system tray.

Built with Rust and Tauri. Available for Windows, macOS, and Linux.

<p align="center">
  <img src="assets/screenshots/dashboard.png" alt="Kagerou dashboard" width="100%" />
</p>

**[Features](#features)** · **[Supported protocols](#supported-protocols)** · **[Getting started](#getting-started)** · **[Development](#development)** · **[Roadmap](ROADMAP.md)** · **[Contributing](CONTRIBUTING.md)**

## Why Kagerou

Most sing-box clients either expose a configuration file or hide the core behind a crowded interface. Kagerou keeps the core's flexibility and gives it a focused desktop UI:

- one dashboard for connection state and traffic;
- profile groups and subscription sources instead of manually editing JSON;
- routing and DNS configuration generated from the settings you choose;
- a bundled, version-pinned sing-box core — no separate installation required;
- native behaviour on each desktop platform, including TUN privileges, autostart, and tray controls.

## Features

### Connection

- TUN mode with platform-specific privilege handling: UAC on Windows, an administrator prompt on macOS, and `CAP_NET_ADMIN` or `pkexec` on Linux.
- Start, stop, and switch profiles without restarting the core.
- Auto-connect to the last active profile on launch.
- System tray controls for connecting, disconnecting, switching recent profiles, showing the window, and quitting.
- Optional launch at login.
- Bundled sing-box core, fetched and SHA-256 verified during the build.

### Profiles and subscriptions

- Profile groups with drag-and-drop ordering.
- Subscription sources loaded from a URL or pasted content.
- Manual and scheduled subscription refresh.
- Per-profile and group-wide connectivity tests.
- Test results, progress reporting, and removal of unavailable profiles.
- Live logs from the sing-box core.

### Routing and observability

- Routing rules for domains, domain suffixes, and IP CIDRs.
- Bypass-LAN and block-ads presets in the UI.
- DNS over HTTPS through the proxy for remote queries.
- Domain sniffing before routing, so domain rules can match connections accurately.
- Download/upload speed, session totals, uptime, connection count, and a live speed chart.
- Optional exit-country and city lookup through `ipwho.is` from inside the tunnel.

### Interface

- English and Russian localisation.
- Catppuccin and Kanagawa themes.
- Dark, light, and system theme variants.
- Native desktop window and tray integration.

Kagerou is actively evolving. Some controls are already visible while their backend behaviour is still being completed. See [ROADMAP.md](ROADMAP.md) for the exact status of every feature.

## Supported protocols

- VLESS
- VMess
- Trojan
- Shadowsocks
- Hysteria2 (`hy2://`)
- TUIC

## Supported subscription formats

- Plain-text or base64-encoded lists of `vless://`, `vmess://`, `trojan://`, `ss://`, `hysteria2://`, and `tuic://` links.
- Clash-style YAML subscriptions with a `proxies:` section.
- sing-box JSON configurations with an `outbounds` section.

## Getting started

Prebuilt bundles are published on the [Releases](https://github.com/binido/Kagerou/releases) page. If you want to run the latest code or build for an unsupported target, build Kagerou locally with Rust, Node.js, and pnpm.

The latest release is [v0.4.0](https://github.com/binido/Kagerou/releases/tag/v0.4.0).

### Prerequisites

- [Rust](https://rustup.rs/) stable, with the platform dependencies required by [Tauri](https://v2.tauri.app/start/prerequisites/);
- [Node.js](https://nodejs.org/) 22 or newer;
- [pnpm](https://pnpm.io/).

### Run in development

Clone the repository and run the Tauri application from its root:

```bash
git clone https://github.com/binido/Kagerou.git
cd Kagerou

pnpm install
pnpm --dir app install
pnpm tauri dev
```

`pnpm tauri dev` downloads the pinned sing-box sidecar automatically before starting the app. The sidecar is stored under `src-tauri/binaries/` and is not committed to the repository.

### Build an installable bundle

```bash
pnpm tauri build
```

The configured bundle targets are:

- macOS: DMG;
- Linux: DEB, RPM, and AppImage;
- Windows: MSI and NSIS installer.

Builds are currently unsigned. macOS and Windows may show a security warning on first launch; this does not mean that the application is corrupted.

## Data and privacy

Kagerou is a local desktop application. Profiles, subscription sources, routing rules, and settings are stored in a local SQLite database in the operating system's application-data directory. The application does not sync this data to a Kagerou server.

When exit-location lookup is enabled, Kagerou asks [ipwho.is](https://ipwho.is/) to identify the address visible through the active tunnel. Disable the lookup in Settings if you do not want this request.

Profile keys are currently stored locally in plain text, alongside the generated sing-box configuration. Protect the operating-system account and its application-data directory accordingly.

## Development

The repository is split into two layers:

- `app/` — React, TypeScript, Vite, Tailwind CSS, shadcn/ui, Zustand, and i18next;
- `src-tauri/` — Rust backend, SQLite storage, subscription parsers, sing-box configuration generation, process supervision, Clash API client, and TUN privilege handling.

Run the checks locally:

```bash
# Rust backend
cd src-tauri
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo test -- --ignored  # smoke test against the real sing-box core

# Frontend
cd ../app
pnpm lint
pnpm test
pnpm build
```

The same checks run in GitHub Actions for every pull request and every push to `main`.

Before changing behaviour, check the corresponding row in [ROADMAP.md](ROADMAP.md). The roadmap is the source of truth for what is complete, partial, planned, or intentionally not supported.

## Roadmap

Kagerou is working towards the feature coverage of [NekoBox for Android](https://github.com/MatsuriDayo/NekoBoxForAndroid), adapted for desktop. Planned and partially implemented areas include configurable DNS and inbound settings, richer routing rules, profile export/import, proxy chains, per-process routing, and an embedded sing-box dashboard.

See the full status and contribution candidates in [ROADMAP.md](ROADMAP.md).

## Credits

- [sing-box](https://github.com/SagerNet/sing-box) — proxy core;
- [Tauri](https://tauri.app/) — native application shell;
- [shadcn/ui](https://ui.shadcn.com/) and [Radix UI](https://www.radix-ui.com/) — interface components;
- [Catppuccin](https://catppuccin.com/) and [Kanagawa](https://github.com/rebelot/kanagawa.nvim) — colour themes.

## Contributing

Bug reports, documentation, design ideas, and pull requests are welcome. Please read [CONTRIBUTING.md](CONTRIBUTING.md) before starting work. For incomplete features, open an issue first so the work can be coordinated and the roadmap stays accurate.

## Contact

- [Open an issue](https://github.com/binido/Kagerou/issues) for bugs and feature requests.

## License

[MIT](LICENSE)
