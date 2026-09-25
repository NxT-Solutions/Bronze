import { readFile } from "node:fs/promises";
import { createServer } from "node:http";
import { extname, join, sep } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(fileURLToPath(new URL(".", import.meta.url)), "..", "src");
const types = {
  ".html": "text/html; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".mjs": "text/javascript; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".svg": "image/svg+xml",
  ".png": "image/png",
  ".json": "application/json",
};

const server = createServer(async (request, response) => {
  const url = new URL(request.url ?? "/", "http://127.0.0.1");
  const rel = decodeURIComponent(url.pathname).replace(/^\/+/, "");
  if (rel.length === 0 || rel.split("/").includes("..")) {
    response.writeHead(403);
    response.end();
    return;
  }
  const path = join(root, rel);
  if (path !== root && !path.startsWith(root + sep)) {
    response.writeHead(403);
    response.end();
    return;
  }
  try {
    const body = await readFile(path);
    response.writeHead(200, {
      "content-type": types[extname(path)] ?? "application/octet-stream",
      "cache-control": "no-store",
    });
    response.end(body);
  } catch {
    response.writeHead(404);
    response.end();
  }
});

server.on("error", (error) => {
  console.error(error);
  process.exit(1);
});

server.listen(4173, "127.0.0.1");
