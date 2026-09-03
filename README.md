# Kagerou

A cleaner, cross-platform [sing-box](https://sing-box.sagernet.org/) VPN client for desktop, built with Tauri.

## Layout

- `app/` — the frontend (React + TypeScript + Vite + Tailwind + shadcn, Zustand for state, i18n via react-i18next).
- `src-tauri/` — the Rust backend: SQLite storage, subscription parsing (vmess/vless/trojan/ss/hysteria2/tuic + Clash/sing-box subscriptions), sing-box config generation, process supervision, the Clash API client, and per-OS TUN privilege handling.

## Prerequisites

- [Rust](https://rustup.rs/) (stable) and [pnpm](https://pnpm.io/)
- A `sing-box` binary on your `PATH` — not bundled in this repo. See [sing-box's install docs](https://sing-box.sagernet.org/installation/).

## Development

```bash
pnpm install
pnpm --dir app install
pnpm tauri dev
```

Run from the repo root (not `app/`) — the Tauri CLI expects `src-tauri/` as a sibling.

## Testing

```bash
cd src-tauri && cargo test    # Rust backend
cd app && pnpm test           # frontend store logic (Vitest)
```

## Building

```bash
pnpm tauri build
```
