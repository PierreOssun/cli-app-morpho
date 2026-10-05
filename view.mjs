// Serves viewer.html and one evidence JSON on localhost, opens the browser, and exits once both are served.
// Usage: node view.mjs [outputs/vault_evidence_….json]   (default: newest file in outputs/)
import { createServer } from "node:http";
import { readFileSync, readdirSync, statSync } from "node:fs";
import { exec } from "node:child_process";

const file = process.argv[2] ?? "outputs/" + readdirSync("outputs").filter((f) => f.endsWith(".json")).sort((a, b) => statSync(`outputs/${b}`).mtimeMs - statSync(`outputs/${a}`).mtimeMs)[0];

const server = createServer((req, res) => {
  const path = decodeURIComponent(new URL(req.url, "http://x").pathname).slice(1);
  if (path === "viewer.html") return res.end(readFileSync("viewer.html"));
  if (path !== file) return res.writeHead(404).end();
  res.writeHead(200, { "content-type": "application/json" }).end(readFileSync(file));
  setTimeout(() => process.exit(0), 1000);
});

server.listen(0, "127.0.0.1", () => {
  const url = `http://127.0.0.1:${server.address().port}/viewer.html?file=${encodeURIComponent(file)}`;
  const open = { darwin: "open", win32: "start \"\"" }[process.platform] ?? "xdg-open";
  exec(`${open} "${url}"`);
  console.log(`Viewer: ${url}`);
});
setTimeout(() => process.exit(0), 60_000); // Edge case: stop anyway if the browser never loads the page.
