import assert from "node:assert/strict";
import test, { beforeEach } from "node:test";

import {
  latestRelease,
  selectPlatformReleases,
  publicDownloadMetadata,
  releaseDownload,
  resetReleaseCache,
} from "./release-downloads.mjs";

const repositoryUrl = "https://github.com/ducnguyen67201/TroApp";
const releasesUrl =
  "https://api.github.com/repos/ducnguyen67201/TroApp/releases";
const macTag = "macos-v0.1.0-59ccc66";
const armDmg = "Tro-0.1.0-macos-arm64.dmg";
const intelDmg = "Tro-0.1.0-macos-x64.dmg";
const windowsInstaller = "Tro-0.1.0-windows-x64-setup.exe";

function githubAsset(name, tag, size = 100) {
  return {
    browser_download_url: `${repositoryUrl}/releases/download/${encodeURIComponent(tag)}/${encodeURIComponent(name)}`,
    name,
    size,
  };
}

function macRelease() {
  return {
    tag_name: macTag,
    draft: false,
    prerelease: false,
    published_at: "2026-10-14T04:00:00Z",
    assets: [
      githubAsset("Tro-0.1.0-macos-arm64.zip", macTag, 110),
      githubAsset(armDmg, macTag, 200),
      githubAsset("Tro-0.1.0-macos-x64.zip", macTag, 120),
      githubAsset(intelDmg, macTag, 250),
    ],
  };
}

function windowsRelease() {
  return {
    tag_name: "v0.1.0",
    draft: false,
    prerelease: false,
    published_at: "2026-10-13T14:53:38Z",
    assets: [githubAsset(windowsInstaller, "v0.1.0", 350)],
  };
}

function jsonResponse(value, status = 200) {
  return new Response(JSON.stringify(value), {
    status,
    headers: { "Content-Type": "application/json" },
  });
}

beforeEach(resetReleaseCache);

test("selects signed Mac DMGs and preserves Windows from its older stable release", () => {
  const release = selectPlatformReleases([macRelease(), windowsRelease()]);
  const metadata = publicDownloadMetadata(release);
  assert.equal(release.version, "0.1.0");
  assert.equal(release.platforms.macosApple.name, armDmg);
  assert.equal(release.platforms.macosIntel.name, intelDmg);
  assert.equal(release.platforms.windows.name, windowsInstaller);
  assert.equal(
    release.platforms.windows.url,
    windowsRelease().assets[0].browser_download_url,
  );
  assert.deepEqual(metadata.platforms.macosApple, {
    channel: "stable",
    format: "dmg",
    href: "/downloads/latest/macos-arm64",
    sizeBytes: 200,
    version: "0.1.0",
  });
  assert.equal(metadata.platforms.macosIntel.format, "dmg");
  assert.equal(metadata.platforms.windows.format, "exe");
  assert.equal(
    releaseDownload(release, "/downloads/latest/macos-x64").name,
    intelDmg,
  );
  assert.equal(
    releaseDownload(release, "/downloads/latest/windows-x64").name,
    windowsInstaller,
  );
  assert.equal(releaseDownload(release, "/downloads/latest/unknown"), null);
});

test("uses a ZIP only when that Mac architecture has no DMG", () => {
  const mac = macRelease();
  mac.assets = mac.assets.filter((asset) => asset.name !== intelDmg);
  const release = selectPlatformReleases([mac]);
  assert.equal(release.platforms.macosApple.format, "dmg");
  assert.equal(release.platforms.macosIntel.format, "zip");
  assert.equal(
    publicDownloadMetadata(release).platforms.macosIntel.sizeBytes,
    120,
  );
  assert.equal(release.platforms.macosIntel.version, "0.1.0");
});

test("selects stable and preview releases independently for every platform", () => {
  const preview = macRelease();
  preview.prerelease = true;
  preview.published_at = "2026-10-15T00:00:00Z";
  preview.assets.push(githubAsset("Tro-0.2.0-windows-x64-setup.exe", macTag));
  const armStable = macRelease();
  armStable.assets = [githubAsset(armDmg, macTag)];
  const release = selectPlatformReleases([
    preview,
    armStable,
    windowsRelease(),
  ]);
  assert.equal(release.platforms.macosApple.channel, "stable");
  assert.equal(release.platforms.macosIntel.channel, "preview");
  assert.equal(release.platforms.windows.channel, "stable");
  assert.equal(release.platforms.windows.version, "0.1.0");
});

test("selects the latest published installer per platform regardless of list order", () => {
  const newerWindows = windowsRelease();
  newerWindows.tag_name = "v0.2.0";
  newerWindows.published_at = "2026-10-16T00:00:00Z";
  newerWindows.assets = [
    githubAsset("Tro-0.2.0-windows-x64-setup.exe", "v0.2.0"),
  ];
  const release = selectPlatformReleases([
    windowsRelease(),
    macRelease(),
    newerWindows,
  ]);
  assert.equal(release.platforms.windows.version, "0.2.0");
  assert.equal(release.platforms.macosApple.version, "0.1.0");
  assert.equal(release.platforms.macosIntel.version, "0.1.0");
  assert.equal(release.version, "0.2.0");
});

test("ignores draft, malformed and unsupported assets without legacy fallback", () => {
  const draft = macRelease();
  draft.draft = true;
  const unsupported = macRelease();
  unsupported.assets = [
    githubAsset("TroCode-darwin-arm64-0.2.0.zip", macTag),
    githubAsset("Tro-0.1.0-macos-ia32.dmg", macTag),
    githubAsset("Tro-0.1.0-windows-arm64-setup.exe", macTag),
    githubAsset(`${armDmg}.blockmap`, macTag),
    githubAsset("MacSignatures-arm64.json", macTag),
    githubAsset("Tro-0.1.0-windows-x64-setup.exe", macTag, 0),
  ];
  const release = selectPlatformReleases([
    null,
    {},
    draft,
    unsupported,
    windowsRelease(),
  ]);
  assert.equal(release.platforms.macosApple, null);
  assert.equal(release.platforms.macosIntel, null);
  assert.equal(release.platforms.windows.version, "0.1.0");
  assert.throws(() => selectPlatformReleases({}), /invalid release-list/);
  assert.throws(
    () => selectPlatformReleases([draft, unsupported]),
    /No downloadable/,
  );
});

for (const invalidUrl of [
  `https://github.com/ducnguyen67201/asdfaefqwefq/releases/download/${macTag}/${armDmg}`,
  `https://github.com/other/TroApp/releases/download/${macTag}/${armDmg}`,
  `https://github.com.example.test/ducnguyen67201/TroApp/releases/download/${macTag}/${armDmg}`,
  `${repositoryUrl}/releases/download/another-tag/${armDmg}`,
  `${repositoryUrl}/releases/download/${macTag}/another-asset.dmg`,
  `${repositoryUrl}/releases/download/${macTag}/${armDmg}?token=private`,
  `${repositoryUrl}/releases/download/${macTag}/${armDmg}#fragment`,
  `http://github.com/ducnguyen67201/TroApp/releases/download/${macTag}/${armDmg}`,
  `https://user:private@github.com/ducnguyen67201/TroApp/releases/download/${macTag}/${armDmg}`,
  `https://github.com:444/ducnguyen67201/TroApp/releases/download/${macTag}/${armDmg}`,
]) {
  test(`rejects untrusted or mismatched asset URL: ${invalidUrl}`, () => {
    const payload = macRelease();
    payload.assets = [
      { ...githubAsset(armDmg, macTag), browser_download_url: invalidUrl },
    ];
    const release = selectPlatformReleases([payload, windowsRelease()]);
    assert.equal(release.platforms.macosApple, null);
    assert.equal(release.platforms.windows.version, "0.1.0");
  });
}

test("fetches and caches one shared TroApp history for all platforms", async () => {
  const requests = [];
  const fetchImplementation = async (url, options) => {
    requests.push(url);
    assert.ok(options.signal instanceof AbortSignal);
    return jsonResponse([macRelease(), windowsRelease()]);
  };
  const first = await latestRelease({ fetchImplementation, now: () => 1_000 });
  const second = await latestRelease({ fetchImplementation, now: () => 1_001 });
  assert.deepEqual(requests, [`${releasesUrl}?per_page=100&page=1`]);
  assert.equal(first, second);
});

test("keeps Intel unavailable until its public asset appears and then refreshes automatically", async () => {
  const armOnly = macRelease();
  armOnly.assets = [githubAsset(armDmg, macTag, 200)];
  const before = await latestRelease({
    fetchImplementation: async () => jsonResponse([armOnly, windowsRelease()]),
    now: () => 1_000,
  });
  assert.equal(before.platforms.macosApple.name, armDmg);
  assert.equal(before.platforms.macosIntel, null);
  assert.equal(before.platforms.windows.name, windowsInstaller);
  const after = await latestRelease({
    fetchImplementation: async () =>
      jsonResponse([macRelease(), windowsRelease()]),
    now: () => 400_000,
  });
  assert.equal(after.platforms.macosIntel.name, intelDmg);
  assert.equal(after.platforms.windows.name, windowsInstaller);
});

test("looks beyond the first release page for an older Windows installer", async () => {
  const empty = { ...macRelease(), assets: [] };
  const requests = [];
  const release = await latestRelease({
    fetchImplementation: async (url) => {
      requests.push(url);
      return jsonResponse(
        url.endsWith("page=1")
          ? [macRelease(), ...Array.from({ length: 99 }, () => empty)]
          : [windowsRelease()],
      );
    },
  });
  assert.equal(requests.length, 2);
  assert.equal(release.platforms.windows.name, windowsInstaller);
  assert.equal(release.platforms.macosApple.name, armDmg);
});

test("shares simultaneous refreshes and retains cached downloads after a GitHub failure", async () => {
  let requests = 0;
  const fetchImplementation = async () => {
    requests += 1;
    return jsonResponse([macRelease(), windowsRelease()]);
  };
  const [first, concurrent] = await Promise.all([
    latestRelease({ fetchImplementation, now: () => 1_000 }),
    latestRelease({ fetchImplementation, now: () => 1_000 }),
  ]);
  assert.equal(requests, 1);
  assert.equal(first, concurrent);
  const cached = await latestRelease({
    fetchImplementation: async () => new Response(null, { status: 503 }),
    now: () => 400_000,
  });
  assert.equal(cached, first);
});

test("rejects unavailable or malformed GitHub responses without an existing cache", async () => {
  await assert.rejects(
    latestRelease({
      fetchImplementation: async () => new Response(null, { status: 503 }),
    }),
    /503/,
  );
  await assert.rejects(
    latestRelease({
      fetchImplementation: async () => jsonResponse({ assets: [] }),
    }),
    /invalid release-list/,
  );
});

test("bounds release pagination without silently hiding older platform installers", async () => {
  let requests = 0;
  await assert.rejects(
    latestRelease({
      fetchImplementation: async () => {
        requests += 1;
        return jsonResponse(
          Array.from({ length: 100 }, () => windowsRelease()),
        );
      },
    }),
    /resolver limit/,
  );
  assert.equal(requests, 10);
});
