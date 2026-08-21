import { createReadStream } from "node:fs";
import { stat } from "node:fs/promises";
import { createServer } from "node:http";
import { extname, join, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";

import {
  latestRelease,
  publicDownloadMetadata,
  releaseDownload,
} from "./release-downloads.mjs";

const appDirectory = fileURLToPath(new URL(".", import.meta.url));
const distDirectory = resolve(appDirectory, "dist");
const port = Number.parseInt(process.env.PORT ?? "4173", 10);
const host = process.env.HOST ?? "0.0.0.0";

const contentTypes = {
  ".css": "text/css; charset=utf-8",
  ".html": "text/html; charset=utf-8",
  ".ico": "image/x-icon",
  ".jpeg": "image/jpeg",
  ".jpg": "image/jpeg",
  ".js": "text/javascript; charset=utf-8",
  ".json": "application/json; charset=utf-8",
  ".mp4": "video/mp4",
  ".png": "image/png",
  ".svg": "image/svg+xml; charset=utf-8",
  ".webp": "image/webp",
  ".zip": "application/zip",
};

const securityHeaders = {
  "Content-Security-Policy":
    "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self'; base-uri 'self'; form-action 'self'; frame-ancestors 'none'",
  "Referrer-Policy": "strict-origin-when-cross-origin",
  "Strict-Transport-Security": "max-age=31536000; includeSubDomains",
  "X-Content-Type-Options": "nosniff",
  "X-Frame-Options": "DENY",
};

function sendJson(response, statusCode, payload) {
  response.writeHead(statusCode, {
    ...securityHeaders,
    "Cache-Control": "no-store",
    "Content-Type": "application/json; charset=utf-8",
  });
  response.end(JSON.stringify(payload));
}

function redirectToDownload(response, location) {
  response.writeHead(302, {
    ...securityHeaders,
    "Cache-Control": "no-store",
    Location: location,
  });
  response.end();
}

function resolvePublicPath(pathname) {
  const relativePath = decodeURIComponent(pathname).replace(/^\/+/, "");
  const candidate = resolve(distDirectory, relativePath || "index.html");
  const isInsideDist =
    candidate === distDirectory ||
    candidate.startsWith(`${distDirectory}${sep}`);

  return isInsideDist ? candidate : null;
}

async function findFile(pathname) {
  const requestedPath = resolvePublicPath(pathname);
  if (!requestedPath) return null;

  try {
    const fileStat = await stat(requestedPath);
    if (fileStat.isDirectory()) return join(requestedPath, "index.html");
    if (fileStat.isFile()) return requestedPath;
  } catch {
    if (!extname(pathname)) return join(distDirectory, "index.html");
  }

  return null;
}

const server = createServer(async (request, response) => {
  if (!request.url || !["GET", "HEAD"].includes(request.method ?? "")) {
    sendJson(response, 405, { error: "method_not_allowed" });
    return;
  }

  const requestUrl = new URL(
    request.url,
    `http://${request.headers.host ?? "localhost"}`,
  );

  if (requestUrl.pathname === "/health") {
    sendJson(response, 200, { status: "ok" });
    return;
  }

  if (requestUrl.pathname === "/api/downloads/latest") {
    try {
      sendJson(response, 200, publicDownloadMetadata(await latestRelease()));
    } catch (error) {
      console.error(
        JSON.stringify({
          error: error instanceof Error ? error.message : "unknown_error",
          event: "release_metadata_failed",
        }),
      );
      sendJson(response, 503, { error: "release_metadata_unavailable" });
    }
    return;
  }

  if (requestUrl.pathname.startsWith("/downloads/latest/")) {
    try {
      const asset = releaseDownload(await latestRelease(), requestUrl.pathname);
      if (!asset) {
        sendJson(response, 404, { error: "release_asset_not_found" });
        return;
      }
      redirectToDownload(response, asset.url);
    } catch (error) {
      console.error(
        JSON.stringify({
          error: error instanceof Error ? error.message : "unknown_error",
          event: "release_download_failed",
        }),
      );
      sendJson(response, 503, { error: "release_download_unavailable" });
    }
    return;
  }

  let filePath;
  try {
    filePath = await findFile(requestUrl.pathname);
  } catch {
    sendJson(response, 400, { error: "invalid_path" });
    return;
  }

  if (!filePath) {
    sendJson(response, 404, { error: "not_found" });
    return;
  }

  const extension = extname(filePath).toLowerCase();
  const isImmutableAsset =
    requestUrl.pathname.startsWith("/assets/") ||
    requestUrl.pathname.startsWith("/demo/") ||
    requestUrl.pathname.startsWith("/downloads/");

  const fileStat = await stat(filePath);
  const fileSize = fileStat.size;
  const rangeHeader = request.headers.range;
  let rangeStart = 0;
  let rangeEnd = Math.max(fileSize - 1, 0);
  let statusCode = 200;

  if (rangeHeader && fileSize > 0) {
    const rangeMatch = /^bytes=(\d*)-(\d*)$/.exec(rangeHeader.trim());
    const startText = rangeMatch?.[1] ?? "";
    const endText = rangeMatch?.[2] ?? "";

    if (!rangeMatch || (!startText && !endText)) {
      response.writeHead(416, {
        ...securityHeaders,
        "Content-Range": `bytes */${fileSize}`,
      });
      response.end();
      return;
    }

    if (!startText) {
      const suffixLength = Number.parseInt(endText, 10);
      if (!Number.isSafeInteger(suffixLength) || suffixLength <= 0) {
        response.writeHead(416, {
          ...securityHeaders,
          "Content-Range": `bytes */${fileSize}`,
        });
        response.end();
        return;
      }
      rangeStart = Math.max(fileSize - suffixLength, 0);
    } else {
      rangeStart = Number.parseInt(startText, 10);
    }

    rangeEnd = endText
      ? Math.min(Number.parseInt(endText, 10), fileSize - 1)
      : fileSize - 1;

    if (
      !Number.isSafeInteger(rangeStart) ||
      !Number.isSafeInteger(rangeEnd) ||
      rangeStart < 0 ||
      rangeStart >= fileSize ||
      rangeEnd < rangeStart
    ) {
      response.writeHead(416, {
        ...securityHeaders,
        "Content-Range": `bytes */${fileSize}`,
      });
      response.end();
      return;
    }

    statusCode = 206;
  }

  const contentLength =
    fileSize === 0 ? 0 : Math.max(rangeEnd - rangeStart + 1, 0);

  response.writeHead(statusCode, {
    ...securityHeaders,
    "Accept-Ranges": "bytes",
    "Cache-Control": isImmutableAsset
      ? "public, max-age=31536000, immutable"
      : "no-cache",
    "Content-Length": String(contentLength),
    ...(statusCode === 206
      ? { "Content-Range": `bytes ${rangeStart}-${rangeEnd}/${fileSize}` }
      : {}),
    "Content-Type": contentTypes[extension] ?? "application/octet-stream",
  });

  if (request.method === "HEAD") {
    response.end();
    return;
  }

  const stream = createReadStream(filePath, {
    start: rangeStart,
    end: rangeEnd,
  });
  stream.on("error", () => {
    if (!response.headersSent)
      sendJson(response, 500, { error: "read_failed" });
    else response.destroy();
  });
  stream.pipe(response);
});

server.listen(port, host, () => {
  console.log(JSON.stringify({ event: "server_started", host, port }));
});

function shutdown(signal) {
  console.log(JSON.stringify({ event: "server_stopping", signal }));
  server.close(() => process.exit(0));
}

process.on("SIGINT", () => shutdown("SIGINT"));
process.on("SIGTERM", () => shutdown("SIGTERM"));
