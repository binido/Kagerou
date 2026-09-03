#!/usr/bin/env node
// Downloads the pinned sing-box release and drops the binary into
// src-tauri/binaries/ under the `sing-box-<target-triple>` name Tauri's
// `bundle.externalBin` expects. The binaries themselves are not committed.
//
// Usage: node scripts/fetch-singbox.mjs [target-triple]
// Defaults to $TAURI_ENV_TARGET_TRIPLE, else the host triple from `rustc -vV`.

import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const VERSION = "1.14.0";

// sha256 of each release archive. Pinned rather than trusting HTTPS alone
// because this binary is later run with administrator/root privileges for TUN
// mode. On a version bump, the mismatch error below prints the hash we got.
const SHA256 = {
  "darwin-arm64": "a150c94012ff768b7261939cd236b9c8554127f45137230295d23a5660225cc9",
  "darwin-amd64": "6cf26fc3501f3117cf781e9405cf5338f60add6da5affae39421af6800ebbcb4",
  "linux-amd64": "2375de6999f4f56ab46b4fc5ddf26a6aba1d3e61a0f4e7ddec2f4690457d5f63",
  "linux-arm64": "04d9b40bc98dc55b6f509ce3292145c65478f65866bea64826ebb2f382385088",
  "windows-amd64": "3ffb56267da14e287be48bd10cf7e6505260125bad940b75101fbb4d5d58e5d6",
  "windows-arm64": "f58dff882b2feb022da8de41943804b38681ecab5e1f490f23602fc37e9d5dd4",
};

const ARCH = { x86_64: "amd64", aarch64: "arm64" };
const OS = { darwin: "darwin", linux: "linux", windows: "windows" };

function fail(message) {
  console.error(`fetch-singbox: ${message}`);
  process.exit(1);
}

function hostTriple() {
  const host = execFileSync("rustc", ["-vV"], { encoding: "utf8" }).match(/^host: (.+)$/m);
  if (!host) fail("could not read the host target triple from `rustc -vV`");
  return host[1];
}

const triple = process.argv[2] || process.env.TAURI_ENV_TARGET_TRIPLE || hostTriple();
// e.g. aarch64-apple-darwin, x86_64-unknown-linux-gnu, x86_64-pc-windows-msvc
const [arch, , os] = triple.split("-");
const platform = OS[os] && ARCH[arch] ? `${OS[os]}-${ARCH[arch]}` : null;
if (!platform) fail(`unsupported target triple: ${triple}`);

const isWindows = os === "windows";
const stem = `sing-box-${VERSION}-${platform}`;
const asset = `${stem}.${isWindows ? "zip" : "tar.gz"}`;

const root = path.resolve(fileURLToPath(import.meta.url), "../..");
const binDir = path.join(root, "src-tauri", "binaries");
const dest = path.join(binDir, `sing-box-${triple}${isWindows ? ".exe" : ""}`);
const stamp = `${dest}.version`;

if (fs.existsSync(dest) && fs.existsSync(stamp) && fs.readFileSync(stamp, "utf8").trim() === VERSION) {
  console.log(`sing-box ${VERSION} already present at ${dest}`);
  process.exit(0);
}

const url = `https://github.com/SagerNet/sing-box/releases/download/v${VERSION}/${asset}`;
console.log(`fetching ${url}`);
const response = await fetch(url);
if (!response.ok) fail(`${url} returned ${response.status} ${response.statusText}`);
const archive = Buffer.from(await response.arrayBuffer());

const digest = createHash("sha256").update(archive).digest("hex");
if (digest !== SHA256[platform]) {
  fail(`checksum mismatch for ${asset}\n  expected ${SHA256[platform]}\n  got      ${digest}`);
}

const work = fs.mkdtempSync(path.join(tmpdir(), "kagerou-singbox-"));
try {
  const archivePath = path.join(work, asset);
  fs.writeFileSync(archivePath, archive);
  // bsdtar/GNU tar ships on every platform we target, and bsdtar (Windows,
  // macOS) reads .zip as happily as .tar.gz.
  execFileSync("tar", ["-xf", archivePath, "-C", work]);
  fs.mkdirSync(binDir, { recursive: true });
  fs.copyFileSync(path.join(work, stem, isWindows ? "sing-box.exe" : "sing-box"), dest);
  fs.chmodSync(dest, 0o755);
  fs.writeFileSync(stamp, `${VERSION}\n`);
} finally {
  fs.rmSync(work, { recursive: true, force: true });
}

console.log(`sing-box ${VERSION} installed at ${dest}`);
