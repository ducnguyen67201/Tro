import assert from "node:assert/strict";
import test from "node:test";

import {
  latestRelease,
  parseLatestRelease,
  parsePreviewReleases,
  publicDownloadMetadata,
  releaseDownload,
  resetReleaseCache,
} from "./release-downloads.mjs";

const githubAsset = (name, size = 100) => ({
  browser_download_url: `https://github.com/ducnguyen67201/TroCode/releases/download/v0.2.0/${encodeURIComponent(name)}`,
  name,
  size,
});

const releasePayload = () => ({
  assets: [
    githubAsset("TroCode-darwin-arm64-0.2.0.zip", 200),
    githubAsset("TroCode-darwin-x64-0.2.0.zip", 250),
    githubAsset("TroCode-0.2.0 Setup.exe", 300),
    githubAsset("TroCode-0.2.0-full.nupkg", 400),
  ],
  draft: false,
  prerelease: false,
  published_at: "2026-08-17T10:00:00Z",
  tag_name: "v0.2.0",
});

test("selects trusted platform installers from the latest release", () => {
  const release = parseLatestRelease(releasePayload());

  assert.equal(release.version, "0.2.0");
  assert.equal(release.platforms.macosApple?.size, 200);
  assert.equal(release.platforms.macosIntel?.size, 250);
  assert.equal(release.platforms.windows?.size, 300);
  assert.equal(
    releaseDownload(release, "/downloads/latest/macos-x64")?.name,
    "TroCode-darwin-x64-0.2.0.zip",
  );
  assert.equal(
    releaseDownload(release, "/downloads/latest/windows-x64")?.name,
    "TroCode-0.2.0 Setup.exe",
  );
  assert.deepEqual(publicDownloadMetadata(release).platforms.windows, {
    channel: "stable",
    href: "/downloads/latest/windows-x64",
    sizeBytes: 300,
    version: "0.2.0",
  });
});

test("ignores untrusted and wrong-architecture release assets", () => {
  const payload = releasePayload();
  payload.assets = [
    {
      browser_download_url: "https://example.com/TroCode Setup.exe",
      name: "TroCode Setup.exe",
      size: 100,
    },
    githubAsset("TroCode-win32-arm64 Setup.exe"),
    githubAsset("TroCode-darwin-ia32-0.2.0.zip"),
  ];

  const release = parseLatestRelease(payload);
  assert.equal(release.platforms.macosApple, null);
  assert.equal(release.platforms.macosIntel, null);
  assert.equal(release.platforms.windows, null);
});

test("caches GitHub release requests", async () => {
  resetReleaseCache();
  let requestCount = 0;
  const fetchImplementation = async () => {
    requestCount += 1;
    return new Response(JSON.stringify(releasePayload()), {
      headers: { "Content-Type": "application/json" },
      status: 200,
    });
  };

  await latestRelease({ fetchImplementation, now: () => 1_000 });
  await latestRelease({ fetchImplementation, now: () => 1_001 });
  assert.equal(requestCount, 1);
});

test("uses a clearly marked unsigned preview when no stable release exists", async () => {
  resetReleaseCache();
  const preview = releasePayload();
  preview.prerelease = true;
  preview.tag_name = "v0.2.0-signpath-bootstrap.14";

  const requests = [];
  const fetchImplementation = async (url) => {
    requests.push(url);
    if (url.endsWith("/latest")) return new Response(null, { status: 404 });
    return new Response(JSON.stringify([preview]), {
      headers: { "Content-Type": "application/json" },
      status: 200,
    });
  };

  const release = await latestRelease({
    fetchImplementation,
    now: () => 2_000,
  });
  const metadata = publicDownloadMetadata(release);

  assert.equal(requests.length, 2);
  assert.equal(metadata.platforms.macosApple?.channel, "unsigned-preview");
  assert.deepEqual(metadata.platforms.macosIntel, {
    channel: "unsigned-preview",
    href: "/downloads/latest/macos-x64",
    sizeBytes: 250,
    version: "0.2.0-signpath-bootstrap.14",
  });
  assert.deepEqual(metadata.platforms.windows, {
    channel: "unsigned-preview",
    href: "/downloads/latest/windows-x64",
    sizeBytes: 300,
    version: "0.2.0-signpath-bootstrap.14",
  });
});

test("ignores drafts and stable entries in the preview release list", () => {
  const draft = releasePayload();
  draft.draft = true;
  draft.prerelease = true;
  const stable = releasePayload();
  const preview = releasePayload();
  preview.prerelease = true;
  preview.tag_name = "v0.3.0-preview.1";

  assert.equal(
    parsePreviewReleases([draft, stable, preview])?.version,
    "0.3.0-preview.1",
  );
});
