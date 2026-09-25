export function installVisualStub(scenario) {
  const items =
    scenario === "populated"
      ? [
          {
            id: "visual-item-1",
            title: "Ship notes",
            body: "Queue the release checklist.",
            contentLanguage: "en",
            sourceAppName: "Notes",
          },
          {
            id: "visual-item-2",
            title: "Accessibility",
            body: "Retest Input Monitoring before capture.",
            contentLanguage: "en",
            sourceAppName: "TextEdit",
          },
        ]
      : [];

  // Unknown commands reject. A resolved clock, path, or random id would
  // fail a shot for a reason other than the screen under test.
  const invoke = (cmd) => {
    if (cmd === "list_queue_items") {
      return Promise.resolve(items);
    }
    if (cmd === "queue_query") {
      return Promise.resolve({ items, nextCursor: null });
    }
    if (cmd === "search_library_items") {
      return Promise.resolve([]);
    }
    return Promise.reject(new Error("visual_stub"));
  };

  globalThis.__TAURI__ = {
    core: { invoke },
    event: {
      listen: () => Promise.resolve(() => {}),
      emit: () => Promise.resolve(),
    },
  };
  globalThis.__TAURI_INTERNALS__ = { invoke };
}
