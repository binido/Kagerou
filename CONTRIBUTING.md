# Contributing to Kagerou

## Picking something up

[ROADMAP.md](ROADMAP.md) lists every feature and its status. It is the source of truth — if the README or a code comment disagrees with it, the roadmap is right.

1. Find a row you want. Rows marked **Good first issue** are small and self-contained; `📋` rows are unstarted; `🟡` rows are half-built and the note says what's missing. A `💡` row is an idea nobody has committed to — say what you think before building it.
2. Open an issue quoting that row, so nobody duplicates the work. Say roughly how you plan to do it if the row leaves the approach open.
3. Send a pull request that updates the row's status in the same commit as the code.

If what you want to build isn't on the roadmap, open an issue first — it may be missing for a reason worth hearing before you write anything.

## Layout

- `app/` — the frontend (React + TypeScript + Vite + Tailwind + shadcn, Zustand for state, i18n via react-i18next).
- `src-tauri/` — the Rust backend: SQLite storage, subscription parsing (vmess/vless/trojan/ss/hysteria2/tuic + Clash/sing-box subscriptions), sing-box config generation, process supervision, the Clash API client, and per-OS TUN privilege handling.

## Prerequisites

- [Rust](https://rustup.rs/) (stable) and [pnpm](https://pnpm.io/)

## The sing-box core

Kagerou bundles sing-box as a [Tauri sidecar](https://tauri.app/develop/sidecar/). The binaries aren't committed — fetch the pinned release for your host first:

```bash
node scripts/fetch-singbox.mjs
```

It lands in `src-tauri/binaries/sing-box-<target-triple>`, which `cargo build` (via `tauri-build`) then copies next to the app executable. Without it the Rust build fails, so run it once after cloning; `pnpm tauri dev` and `pnpm tauri build` run it for you.

Pass a target triple to fetch for another platform (`node scripts/fetch-singbox.mjs x86_64-pc-windows-msvc`) — that's what a cross-platform release build needs. Bumping the version means editing `VERSION` and the pinned checksums at the top of the script; the mismatch error prints the hash it actually got.

## Development

```bash
pnpm install
pnpm --dir app install
pnpm tauri dev
```

Run from the repo root (not `app/`) — the Tauri CLI expects `src-tauri/` as a sibling.

## Testing

```bash
cd src-tauri && cargo test              # Rust backend
cd src-tauri && cargo test -- --ignored # plus the smoke test against the real sing-box binary
cd app && pnpm test                     # frontend store logic (Vitest)
```

New backend logic should ship with tests covering edge cases and error paths, not just the happy path — see the existing `#[cfg(test)] mod tests` blocks in `src-tauri/src/` for the pattern (mocked launchers/HTTP servers instead of touching a real sing-box process). Frontend tests target the Zustand store's business logic, not component rendering.

## Code style

- Rust: run `cargo fmt` and `cargo clippy --all-targets` before committing; both should be clean.
- Frontend: `pnpm lint` (oxlint) and `pnpm build` (runs `tsc -b`) should both be clean.

## What CI checks

Every pull request runs the commands above — `cargo fmt --check`, `cargo clippy --all-targets -D warnings`, `cargo test` (plus the `--ignored` smoke test against the real sing-box binary), and `pnpm lint` / `test` / `build`. Run them locally first; the workflow is `.github/workflows/ci.yml`.

The backend job runs on Linux only. The per-OS privilege logic takes the target OS as an argument rather than being `cfg`-gated, so all three branches are exercised from a single host.

## Building

```bash
pnpm tauri build
```

## Cutting a release

1. Bump `version` in `src-tauri/tauri.conf.json`. It is the only place the version lives — Vite injects it into the frontend, and the release workflow refuses a tag that disagrees with it.
2. Tag and push: `git tag v0.2.0 && git push origin v0.2.0`.
3. The workflow builds macOS (Apple silicon and Intel), Linux and Windows bundles and attaches them to a **draft** release. Check the artifacts, write the notes, then publish it yourself.

Builds are unsigned, so macOS and Windows will warn on first launch. A tag with a pre-release suffix (`v0.2.0-alpha.1`) is marked as a pre-release; GitHub's "latest release" endpoint skips those and drafts alike, which is also what the in-app update check reads.

## Submitting changes

Keep commits small and logical — one coherent change per commit, not a single commit bundling unrelated work. Open a PR against `main` with a description of what changed and why.

A change that alters behaviour also updates its row in [ROADMAP.md](ROADMAP.md), in the same commit. Don't leave it as a follow-up, and don't move a row to `✅` because the UI looks right — `🟡` exists so that half-finished work stays visible.
