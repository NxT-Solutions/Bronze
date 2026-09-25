import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const desktop = dirname(fileURLToPath(import.meta.url));
const repo = join(desktop, "../../..");
const tauriDir = join(desktop, "../src-tauri");

const SHIPPED_LOCALES = ["en", "nl", "fr", "de", "es", "it", "en-XA", "ar-XB"];
const REQUIRED_KEYS = [
  "app.name",
  "settings.title",
  "queue.item.complete",
  "settings.permission.retest",
];

test("packaged desktop entry points exist and locale catalogs load", () => {
  const conf = JSON.parse(
    readFileSync(join(tauriDir, "tauri.conf.json"), "utf8"),
  );
  assert.equal(conf.productName, "Bronze");
  assert.equal(conf.identifier, "app.bronze.desktop");
  assert.equal(conf.build.frontendDist, "../src");
  assert.deepEqual(
    conf.app.windows.map((window) => window.label),
    ["quick", "library", "settings", "help"],
  );
  for (const window of conf.app.windows) {
    assert.equal(existsSync(join(desktop, window.url)), true, window.url);
  }
  assert.equal(
    conf.bundle.resources["../../../packages/i18n/locales"],
    "locales/",
  );
  assert.equal(
    conf.bundle.macOS.entitlements,
    "entitlements/macos.release.plist",
  );
  const entitlements = readFileSync(
    join(tauriDir, conf.bundle.macOS.entitlements),
    "utf8",
  );
  assert.equal(JSON.stringify(conf).includes("get-task-allow"), false);
  assert.equal(entitlements.includes("get-task-allow"), false);
  assert.equal(existsSync(join(tauriDir, "icons/icon.icns")), true);

  const packageSwift = readFileSync(
    join(repo, "native/macos/BronzeNative/Package.swift"),
    "utf8",
  );
  assert.match(packageSwift, /name: "BronzeNative", type: \.static/);
  assert.match(packageSwift, /name: "BronzeNotice"/);
  assert.match(packageSwift, /name: "BronzeNativeTests"/);

  for (const locale of SHIPPED_LOCALES) {
    const catalog = JSON.parse(
      readFileSync(
        join(repo, "packages/i18n/locales", locale, "app.json"),
        "utf8",
      ),
    );
    for (const key of REQUIRED_KEYS) {
      assert.equal(typeof catalog[key], "string", `${locale} ${key}`);
      assert.notEqual(catalog[key], "", `${locale} ${key}`);
    }
  }
  assert.equal(
    existsSync(join(repo, "packages/i18n/locales/zz/app.json")),
    false,
  );
});
