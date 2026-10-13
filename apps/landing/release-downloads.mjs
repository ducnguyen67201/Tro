/** @typedef {{ repository: string, previewChannel: string, platforms: string[] }} ReleaseSource */

/** @type {ReleaseSource} */
const macReleaseSource = {
  repository: "ducnguyen67201/asdfaefqwefq",
  previewChannel: "unsigned-preview",
  platforms: ["macosApple", "macosIntel"],
};

/** @type {ReleaseSource} */
const windowsReleaseSource = {
  repository: "ducnguyen67201/TroApp",
  previewChannel: "preview",
  platforms: ["windows"],
};
const CACHE_TTL_MS = 5 * 60 * 1000;

const downloadRoutes = {
  macosApple: "/downloads/latest/macos-arm64",
  macosIntel: "/downloads/latest/macos-x64",
  windows: "/downloads/latest/windows-x64",
};

let cachedRelease = null;
let cacheExpiresAt = 0;
let pendingRequest = null;

function isRecord(value) {
  return Boolean(value) && typeof value === "object" && !Array.isArray(value);
}

function releaseAsset(value, source) {
  if (!isRecord(value)) return null;
  if (
    typeof value.name !== "string" ||
    typeof value.browser_download_url !== "string" ||
    typeof value.size !== "number" ||
    !Number.isSafeInteger(value.size) ||
    value.size < 0
  ) {
    return null;
  }

  let downloadUrl;
  try {
    downloadUrl = new URL(value.browser_download_url);
  } catch {
    return null;
  }

  if (
    downloadUrl.protocol !== "https:" ||
    downloadUrl.hostname !== "github.com" ||
    !downloadUrl.pathname
      .toLowerCase()
      .startsWith(`/${source.repository}/releases/download/`.toLowerCase())
  ) {
    return null;
  }

  return {
    name: value.name,
    size: value.size,
    url: downloadUrl.toString(),
  };
}

function findMacOSAsset(assets, architecture) {
  return assets.find((asset) => {
    const name = asset.name.toLowerCase();
    return (
      name.endsWith(".zip") &&
      name.includes(architecture) &&
      ["darwin", "mac", "osx"].some((platform) => name.includes(platform))
    );
  });
}

function findWindowsAsset(assets) {
  return assets.find((asset) => {
    const name = asset.name.toLowerCase();
    return (
      name.endsWith(".exe") &&
      name.includes("setup") &&
      !name.includes("arm64") &&
      !name.includes("ia32")
    );
  });
}

function parseRelease(value, expectedPrerelease, source) {
  if (
    !isRecord(value) ||
    value.draft === true ||
    value.prerelease !== expectedPrerelease ||
    typeof value.tag_name !== "string" ||
    !Array.isArray(value.assets)
  ) {
    return null;
  }

  const assets = value.assets
    .map((asset) => releaseAsset(asset, source))
    .filter(Boolean);
  const version = value.tag_name.trim().replace(/^v/, "");
  if (!version) return null;
  const macosAppleAsset = findMacOSAsset(assets, "arm64") ?? null;
  const macosIntelAsset = findMacOSAsset(assets, "x64") ?? null;
  const windowsAsset = findWindowsAsset(assets) ?? null;
  const releaseTarget = (asset) =>
    asset
      ? {
          ...asset,
          channel: expectedPrerelease ? source.previewChannel : "stable",
          version,
        }
      : null;

  return {
    platforms: {
      macosApple: releaseTarget(macosAppleAsset),
      macosIntel: releaseTarget(macosIntelAsset),
      windows: releaseTarget(windowsAsset),
    },
    publishedAt:
      typeof value.published_at === "string" ? value.published_at : null,
    version,
  };
}

export function parseLatestRelease(value, source = macReleaseSource) {
  const release = parseRelease(value, false, source);
  if (!release) {
    throw new Error("GitHub returned invalid latest-release metadata.");
  }
  return release;
}

export function parsePreviewReleases(value, source = macReleaseSource) {
  if (!Array.isArray(value)) {
    throw new Error("GitHub returned invalid release-list metadata.");
  }

  for (const candidate of value) {
    const release = parseRelease(candidate, true, source);
    if (source.platforms.some((platform) => release?.platforms[platform])) {
      return release;
    }
  }

  return null;
}

function mergeReleaseChannels(stable, preview) {
  if (!stable && !preview) {
    throw new Error("No downloadable TroCode release is available.");
  }

  return {
    platforms: {
      macosApple:
        stable?.platforms.macosApple ?? preview?.platforms.macosApple ?? null,
      macosIntel:
        stable?.platforms.macosIntel ?? preview?.platforms.macosIntel ?? null,
      windows: stable?.platforms.windows ?? preview?.platforms.windows ?? null,
    },
    publishedAt: stable?.publishedAt ?? preview?.publishedAt ?? null,
    version: stable?.version ?? preview?.version ?? null,
  };
}

export function publicDownloadMetadata(release) {
  return {
    platforms: {
      macosApple: release.platforms.macosApple
        ? {
            channel: release.platforms.macosApple.channel,
            href: downloadRoutes.macosApple,
            sizeBytes: release.platforms.macosApple.size,
            version: release.platforms.macosApple.version,
          }
        : null,
      macosIntel: release.platforms.macosIntel
        ? {
            channel: release.platforms.macosIntel.channel,
            href: downloadRoutes.macosIntel,
            sizeBytes: release.platforms.macosIntel.size,
            version: release.platforms.macosIntel.version,
          }
        : null,
      windows: release.platforms.windows
        ? {
            channel: release.platforms.windows.channel,
            href: downloadRoutes.windows,
            sizeBytes: release.platforms.windows.size,
            version: release.platforms.windows.version,
          }
        : null,
    },
    publishedAt: release.publishedAt,
    version: release.version,
  };
}

async function requestSourceRelease(fetchImplementation, source) {
  const releasesUrl = `https://api.github.com/repos/${source.repository}/releases`;
  const headers = {
    Accept: "application/vnd.github+json",
    "User-Agent": "Tro-landing-download-resolver",
    "X-GitHub-Api-Version": "2022-11-28",
  };
  if (process.env.GITHUB_TOKEN) {
    headers.Authorization = `Bearer ${process.env.GITHUB_TOKEN}`;
  }

  const stableResponse = await fetchImplementation(`${releasesUrl}/latest`, {
    headers,
  });
  let stable = null;
  if (stableResponse.ok) {
    stable = parseLatestRelease(await stableResponse.json(), source);
    if (source.platforms.every((platform) => stable.platforms[platform])) {
      return stable;
    }
  } else if (stableResponse.status !== 404) {
    throw new Error(
      `GitHub latest-release request failed with ${stableResponse.status}.`,
    );
  }

  const releasesResponse = await fetchImplementation(
    `${releasesUrl}?per_page=20`,
    {
      headers,
    },
  );
  if (!releasesResponse.ok) {
    throw new Error(
      `GitHub release-list request failed with ${releasesResponse.status}.`,
    );
  }

  return mergeReleaseChannels(
    stable,
    parsePreviewReleases(await releasesResponse.json(), source),
  );
}

/** Keep each platform tied to its release owner; never fall back to legacy Windows builds. */
async function requestLatestRelease(fetchImplementation) {
  const [macRelease, windowsRelease] = await Promise.all([
    requestSourceRelease(fetchImplementation, macReleaseSource),
    requestSourceRelease(fetchImplementation, windowsReleaseSource),
  ]);
  return {
    platforms: {
      macosApple: macRelease.platforms.macosApple,
      macosIntel: macRelease.platforms.macosIntel,
      windows: windowsRelease.platforms.windows,
    },
    publishedAt: windowsRelease.publishedAt,
    version: windowsRelease.version,
  };
}

export async function latestRelease({
  fetchImplementation = fetch,
  now = Date.now,
} = {}) {
  const timestamp = now();
  if (cachedRelease && timestamp < cacheExpiresAt) return cachedRelease;
  if (pendingRequest) return pendingRequest;

  pendingRequest = requestLatestRelease(fetchImplementation)
    .then((release) => {
      cachedRelease = release;
      cacheExpiresAt = timestamp + CACHE_TTL_MS;
      return release;
    })
    .catch((error) => {
      if (cachedRelease) return cachedRelease;
      throw error;
    })
    .finally(() => {
      pendingRequest = null;
    });

  return pendingRequest;
}

export function resetReleaseCache() {
  cachedRelease = null;
  cacheExpiresAt = 0;
  pendingRequest = null;
}

export function releaseDownload(release, pathname) {
  if (pathname === downloadRoutes.macosApple) {
    return release.platforms.macosApple;
  }
  if (pathname === downloadRoutes.macosIntel) {
    return release.platforms.macosIntel;
  }
  if (pathname === downloadRoutes.windows) return release.platforms.windows;
  return null;
}
