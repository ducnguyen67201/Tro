import { createReadStream } from "node:fs";
import { stat } from "node:fs/promises";
import { createServer } from "node:http";
import { extname, join, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";

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
  ".png": "image/png",
  ".svg": "image/svg+xml; charset=utf-8",
  ".webp": "image/webp",
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
  const isImmutableAsset = requestUrl.pathname.startsWith("/assets/");

  response.writeHead(200, {
    ...securityHeaders,
    "Cache-Control": isImmutableAsset
      ? "public, max-age=31536000, immutable"
      : "no-cache",
    "Content-Type": contentTypes[extension] ?? "application/octet-stream",
  });

  if (request.method === "HEAD") {
    response.end();
    return;
  }

  const stream = createReadStream(filePath);
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
