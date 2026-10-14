/** @typedef {"macosApple" | "macosIntel" | "windows"} DownloadPlatform */
/** @typedef {"dmg" | "zip" | "exe"} DownloadFormat */
/** @typedef {{ name: string, size: number, url: string }} ReleaseAsset */
/** @typedef {ReleaseAsset & { version: string, format: DownloadFormat }} Installer */
/** @typedef {Installer & { channel: "stable" | "preview", publishedAt: string | null }} DownloadTarget */
/** @typedef {{ platforms: Record<DownloadPlatform, DownloadTarget | null>, publishedAt: string | null, version: string | null }} DownloadRelease */

const releaseRepository = "ducnguyen67201/TroApp";
const CACHE_TTL_MS = 5 * 60 * 1000;
const RELEASES_PER_PAGE = 100;
const MAX_RELEASE_PAGES = 10;

const downloadRoutes = {
  macosApple: "/downloads/latest/macos-arm64",
  macosIntel: "/downloads/latest/macos-x64",
  windows: "/downloads/latest/windows-x64",
};
/** @type {DownloadPlatform[]} */
const platforms = ["macosApple", "macosIntel", "windows"];

/** @type {DownloadRelease | null} */
let cachedRelease = null;
let cacheExpiresAt = 0;
/** @type {Promise<DownloadRelease> | null} */
let pendingRequest = null;

/** @param {unknown} value @returns {value is Record<string, unknown>} */
function isRecord(value) {
  return Boolean(value) && typeof value === "object" && !Array.isArray(value);
}

/** Accept only this release's exact asset URL in the signed app repository. */
/** @param {unknown} value @param {string} tag @returns {ReleaseAsset | null} */
function releaseAsset(value, tag) {
  if (!isRecord(value)) {
    return null;
  }

  if (
    typeof value.name !== "string" ||
    typeof value.browser_download_url !== "string" ||
    typeof value.size !== "number" ||
    !Number.isSafeInteger(value.size) ||
    value.size <= 0
  ) {
    return null;
  }

  let downloadUrl;
  try {
    downloadUrl = new URL(value.browser_download_url);
  } catch {
    return null;
  }
  const expectedPath = `/${releaseRepository}/releases/download/${encodeURIComponent(tag)}/${encodeURIComponent(value.name)}`;
  if (
    downloadUrl.origin !== "https://github.com" ||
    downloadUrl.username ||
    downloadUrl.password ||
    downloadUrl.search ||
    downloadUrl.hash ||
    downloadUrl.pathname !== expectedPath
  ) {
    return null;
  }
  return { name: value.name, size: value.size, url: downloadUrl.toString() };
}

/** Artifact versions stay correct even when a release tag is platform-qualified. */
/** @param {ReleaseAsset[]} assets @param {"arm64" | "x64"} architecture @returns {Installer | null} */
function findMacOSAsset(assets, architecture) {
  for (const format of /** @type {const} */ (["dmg", "zip"])) {
    for (const asset of assets) {
      const match =
        /^Tro-(\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?)-macos-(arm64|x64)\.(dmg|zip)$/.exec(
          asset.name,
        );
      if (match?.[1] && match[2] === architecture && match[3] === format) {
        return { ...asset, format, version: match[1] };
      }
    }
  }
  return null;
}

/** @param {ReleaseAsset[]} assets @returns {Installer | null} */
function findWindowsAsset(assets) {
  for (const asset of assets) {
    const match =
      /^Tro-(\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?)-windows-x64-setup\.exe$/.exec(
        asset.name,
      );
    if (match?.[1]) {
      return { ...asset, format: "exe", version: match[1] };
    }
  }
  return null;
}

/** @param {unknown} value @returns {DownloadRelease | null} */
function parseRelease(value) {
  if (
    !isRecord(value) ||
    value.draft !== false ||
    typeof value.prerelease !== "boolean" ||
    typeof value.tag_name !== "string" ||
    !value.tag_name.trim() ||
    !Array.isArray(value.assets)
  ) {
    return null;
  }
  const assets = value.assets
    .map((asset) => releaseAsset(asset, value.tag_name))
    .filter((asset) => asset !== null);
  const publishedAt =
    typeof value.published_at === "string" &&
    Number.isFinite(Date.parse(value.published_at))
      ? value.published_at
      : null;
  /** @param {Installer | null} asset @returns {DownloadTarget | null} */
  const releaseTarget = (asset) =>
    asset
      ? {
          ...asset,
          channel: value.prerelease ? "preview" : "stable",
          publishedAt,
        }
      : null;
  return {
    platforms: {
      macosApple: releaseTarget(findMacOSAsset(assets, "arm64")),
      macosIntel: releaseTarget(findMacOSAsset(assets, "x64")),
      windows: releaseTarget(findWindowsAsset(assets)),
    },
    publishedAt,
    version: null,
  };
}

/** Select each platform independently: its newest stable installer, then its newest preview. */
/** @param {unknown} value @returns {DownloadRelease} */
export function selectPlatformReleases(value) {
  if (!Array.isArray(value)) {
    throw new Error("GitHub returned invalid release-list metadata.");
  }
  const releases = value
    .map(parseRelease)
    .filter((release) => release !== null);
  releases.sort(
    (left, right) =>
      Date.parse(right.publishedAt ?? "1970-01-01") -
      Date.parse(left.publishedAt ?? "1970-01-01"),
  );
  /** @type {DownloadRelease} */
  const selected = {
    platforms: { macosApple: null, macosIntel: null, windows: null },
    publishedAt: null,
    version: null,
  };
  for (const channel of ["stable", "preview"]) {
    for (const release of releases) {
      for (const platform of platforms) {
        const asset = release.platforms[platform];
        if (!selected.platforms[platform] && asset?.channel === channel) {
          selected.platforms[platform] = asset;
        }
      }
    }
  }
  const newest = Object.values(selected.platforms)
    .filter((asset) => asset !== null)
    .sort(
      (left, right) =>
        Date.parse(right.publishedAt ?? "1970-01-01") -
        Date.parse(left.publishedAt ?? "1970-01-01"),
    )[0];
  if (!newest) {
    throw new Error("No downloadable Tro release is available.");
  }

  selected.publishedAt = newest.publishedAt;
  selected.version = newest.version;
  return selected;
}

/** @param {DownloadRelease} release */
export function publicDownloadMetadata(release) {
  /** @param {DownloadPlatform} platform */
  const metadata = (platform) => {
    const asset = release.platforms[platform];
    return asset
      ? {
          channel: asset.channel,
          format: asset.format,
          href: downloadRoutes[platform],
          sizeBytes: asset.size,
          version: asset.version,
        }
      : null;
  };
  return {
    platforms: {
      macosApple: metadata("macosApple"),
      macosIntel: metadata("macosIntel"),
      windows: metadata("windows"),
    },
    publishedAt: release.publishedAt,
    version: release.version,
  };
}

/** Fetch one shared history so a newer Mac-only release cannot hide an older Windows installer. */
/** @param {typeof fetch} fetchImplementation @returns {Promise<DownloadRelease>} */
async function requestLatestRelease(fetchImplementation) {
  const releasesUrl = `https://api.github.com/repos/${releaseRepository}/releases`;
  const headers = {
    Accept: "application/vnd.github+json",
    "User-Agent": "Tro-landing-download-resolver",
    "X-GitHub-Api-Version": "2022-11-28",
  };
  if (process.env.GITHUB_TOKEN) {
    headers.Authorization = `Bearer ${process.env.GITHUB_TOKEN}`;
  }

  /** @type {unknown[]} */
  const releases = [];
  for (let page = 1; page <= MAX_RELEASE_PAGES; page += 1) {
    const response = await fetchImplementation(
      `${releasesUrl}?per_page=${RELEASES_PER_PAGE}&page=${page}`,
      { headers, signal: AbortSignal.timeout(10_000) },
    );
    if (!response.ok) {
      throw new Error(
        `GitHub release-list request failed with ${response.status}.`,
      );
    }
    const value = await response.json();
    if (!Array.isArray(value)) {
      throw new Error("GitHub returned invalid release-list metadata.");
    }

    releases.push(...value);
    if (value.length < RELEASES_PER_PAGE) {
      return selectPlatformReleases(releases);
    }
  }
  throw new Error(
    "GitHub release history exceeded the download resolver limit.",
  );
}

/** @param {{ fetchImplementation?: typeof fetch, now?: () => number }} options */
export async function latestRelease({
  fetchImplementation = fetch,
  now = Date.now,
} = {}) {
  const timestamp = now();
  if (cachedRelease && timestamp < cacheExpiresAt) {
    return cachedRelease;
  }

  if (pendingRequest) {
    return pendingRequest;
  }

  pendingRequest = requestLatestRelease(fetchImplementation)
    .then((release) => {
      cachedRelease = release;
      cacheExpiresAt = timestamp + CACHE_TTL_MS;
      return release;
    })
    .catch((error) => {
      if (cachedRelease) {
        return cachedRelease;
      }

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

/** @param {DownloadRelease} release @param {string} pathname @returns {DownloadTarget | null} */
export function releaseDownload(release, pathname) {
  for (const platform of platforms) {
    if (pathname === downloadRoutes[platform]) {
      return release.platforms[platform];
    }
  }
  return null;
}
