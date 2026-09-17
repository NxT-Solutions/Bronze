import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  ADR_018_STATUS,
  announceCount,
  applyLibraryEmptyCopy,
  libraryEmptyKey,
  librarySection,
  QUE_007_COMPLETE,
  syncLibraryNav,
} from "./library-live.mjs";

const root = dirname(fileURLToPath(import.meta.url));
const live = readFileSync(join(root, "library-live.mjs"), "utf8");
const html = readFileSync(join(root, "library.html"), "utf8");

test("library search stays a substring placeholder", () => {
  assert.equal(announceCount(0), "0 items");
  assert.equal(announceCount(2), "2 items");
  assert.equal(QUE_007_COMPLETE, false);
  assert.equal(ADR_018_STATUS, "Proposed");
});

test("library items sanitize markdown and keep a title plus expand reader", () => {
  assert.match(live, /fillItemChrome/);
  assert.doesNotMatch(live, /body\.textContent = item\.body/);
  assert.doesNotMatch(live, /innerHTML = item\.body/);
  assert.match(html, /id="library-item-template"/);
  assert.match(html, /data-slot="title"/);
  assert.match(html, /data-slot="expand"/);
  assert.match(html, /data-i18n="queue.item.showMore"/);
  assert.match(html, /data-i18n="queue.item.showLess"/);
  assert.match(html, /data-slot="source"/);
  assert.match(html, /data-slot="source-icon"/);
  assert.match(live, /applySourceRow/);
  assert.match(live, /sourceAppIcon/);
  assert.match(live, /runBusy/);
  assert.match(live, /is-entering/);
  assert.match(live, /hashchange/);
  assert.match(live, /section === "search" && query/);
  assert.match(html, /aria-current="page"/);
});

test("library segment follows the hash and marks the current page", () => {
  assert.equal(librarySection(""), "archive");
  assert.equal(librarySection("#archive"), "archive");
  assert.equal(librarySection("#trash"), "trash");
  assert.equal(librarySection("#search"), "search");
  const archive = {
    href: "#archive",
    current: null,
    getAttribute(name) {
      return name === "href" ? "#archive" : this.current;
    },
    setAttribute(name, value) {
      if (name === "aria-current") {
        this.current = value;
      }
    },
    removeAttribute(name) {
      if (name === "aria-current") {
        this.current = null;
      }
    },
  };
  const trash = {
    href: "#trash",
    current: "page",
    getAttribute(name) {
      return name === "href" ? "#trash" : this.current;
    },
    setAttribute(name, value) {
      if (name === "aria-current") {
        this.current = value;
      }
    },
    removeAttribute(name) {
      if (name === "aria-current") {
        this.current = null;
      }
    },
  };
  const section = syncLibraryNav(
    {
      location: { hash: "#archive" },
      querySelectorAll() {
        return [archive, trash];
      },
    },
    "#archive",
  );
  assert.equal(section, "archive");
  assert.equal(archive.current, "page");
  assert.equal(trash.current, null);
});

test("library empty copy names the next action for each section", () => {
  assert.equal(libraryEmptyKey("archive"), "library.state.empty");
  assert.equal(libraryEmptyKey("trash"), "library.state.emptyTrash");
  assert.equal(libraryEmptyKey("search"), "library.state.emptySearch");
  const title = { textContent: "" };
  const sources = {
    "library.state.empty": {
      textContent: "Capture or type in the queue to fill the archive.",
    },
    "library.state.emptyTrash": { textContent: "Trash is empty." },
  };
  const root = {
    querySelector(sel) {
      if (sel === "#library-empty-title") {
        return title;
      }
      const key = sel.match(/data-i18n="([^"]+)"/)?.[1];
      return sources[key] ?? null;
    },
  };
  assert.equal(
    applyLibraryEmptyCopy(root, "trash"),
    "library.state.emptyTrash",
  );
  assert.equal(title.textContent, "Trash is empty.");
  assert.match(html, /data-empty-message/);
  assert.match(html, /Export…/);
  assert.match(html, /Import…/);
});
