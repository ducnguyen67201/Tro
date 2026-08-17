const GITHUB_API_URL =
  "https://api.github.com/repos/ducnguyen67201/TroCode/releases/latest";
const GITHUB_RELEASES_API_URL =
  "https://api.github.com/repos/ducnguyen67201/TroCode/releases?per_page=20";
const RELEASE_DOWNLOAD_PATH =
  "/ducnguyen67201/TroCode/releases/download/".toLowerCase();
const CACHE_TTL_MS = 5 * 60 * 1000;

const downloadRoutes = {
  macos: "/downloads/latest/macos-arm64",
  windows: "/downloads/latest/windows-x64",
};

let cachedRelease = null;
let cacheExpiresAt = 0;
let pendingRequest = null;

function isRecord(value) {
  return Boolean(value) && typeof value === "object" && !Array.isArray(value);
}

function releaseAsset(value) {
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
    !downloadUrl.pathname.toLowerCase().startsWith(RELEASE_DOWNLOAD_PATH)
  ) {
    return null;
  }

  return {
    name: value.name,
    size: value.size,
    url: downloadUrl.toString(),
  };
}

function findMacOSAsset(assets) {
  return assets.find((asset) => {
    const name = asset.name.toLowerCase();
    return (
      name.endsWith(".zip") &&
      name.includes("arm64") &&
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

function parseRelease(value, expectedPrerelease) {
  if (
    !isRecord(value) ||
    value.draft === true ||
    value.prerelease !== expectedPrerelease ||
    typeof value.tag_name !== "string" ||
    !Array.isArray(value.assets)
  ) {
    return null;
  }

  const assets = value.assets.map(releaseAsset).filter(Boolean);
  const version = value.tag_name.trim().replace(/^v/, "");
  if (!version) return null;
  const macosAsset = findMacOSAsset(assets) ?? null;
  const windowsAsset = findWindowsAsset(assets) ?? null;

  return {
    platforms: {
      macos: macosAsset
        ? {
            ...macosAsset,
            channel: expectedPrerelease ? "unsigned-preview" : "stable",
            version,
          }
        : null,
      windows: windowsAsset
        ? {
            ...windowsAsset,
            channel: expectedPrerelease ? "unsigned-preview" : "stable",
            version,
          }
        : null,
    },
    publishedAt:
      typeof value.published_at === "string" ? value.published_at : null,
    version,
  };
}

export function parseLatestRelease(value) {
  const release = parseRelease(value, false);
  if (!release) {
    throw new Error("GitHub returned invalid latest-release metadata.");
  }
  return release;
}

export function parsePreviewReleases(value) {
  if (!Array.isArray(value)) {
    throw new Error("GitHub returned invalid release-list metadata.");
  }

  for (const candidate of value) {
    const release = parseRelease(candidate, true);
    if (release?.platforms.macos || release?.platforms.windows) return release;
  }

  return null;
}

function mergeReleaseChannels(stable, preview) {
  if (!stable && !preview) {
    throw new Error("No downloadable TroCode release is available.");
  }

  return {
    platforms: {
      macos: stable?.platforms.macos ?? preview?.platforms.macos ?? null,
      windows: stable?.platforms.windows ?? preview?.platforms.windows ?? null,
    },
    publishedAt: stable?.publishedAt ?? preview?.publishedAt ?? null,
    version: stable?.version ?? preview?.version ?? null,
  };
}

export function publicDownloadMetadata(release) {
  return {
    platforms: {
      macos: release.platforms.macos
        ? {
            channel: release.platforms.macos.channel,
            href: downloadRoutes.macos,
            sizeBytes: release.platforms.macos.size,
            version: release.platforms.macos.version,
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

async function requestLatestRelease(fetchImplementation) {
  const headers = {
    Accept: "application/vnd.github+json",
    "User-Agent": "Tro-landing-download-resolver",
    "X-GitHub-Api-Version": "2022-11-28",
  };
  if (process.env.GITHUB_TOKEN) {
    headers.Authorization = `Bearer ${process.env.GITHUB_TOKEN}`;
  }

  const stableResponse = await fetchImplementation(GITHUB_API_URL, { headers });
  let stable = null;
  if (stableResponse.ok) {
    stable = parseLatestRelease(await stableResponse.json());
    if (stable.platforms.macos && stable.platforms.windows) return stable;
  } else if (stableResponse.status !== 404) {
    throw new Error(
      `GitHub latest-release request failed with ${stableResponse.status}.`,
    );
  }

  const releasesResponse = await fetchImplementation(GITHUB_RELEASES_API_URL, {
    headers,
  });
  if (!releasesResponse.ok) {
    throw new Error(
      `GitHub release-list request failed with ${releasesResponse.status}.`,
    );
  }

  return mergeReleaseChannels(
    stable,
    parsePreviewReleases(await releasesResponse.json()),
  );
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
  if (pathname === downloadRoutes.macos) return release.platforms.macos;
  if (pathname === downloadRoutes.windows) return release.platforms.windows;
  return null;
}
