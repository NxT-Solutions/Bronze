## 2026-09-16T12:33:05.280Z 2-2-RUST-MACOS-FACADE-DOCS

hedgehog verify git add fails on Unicode story path 2-2-rust-macos-façade.md (escaped as \303\247). Renamed to ASCII 2-2-rust-macos-facade.md.

## 2026-09-16T13:26:47.419Z 5-4-NATIVE-DISPLAY-PREFERENCE-BRIDGE-JOIN

Pre-writing .hedgehog/noop/<task>.json before hedgehog verify makes join fail ('already recorded as a no-op') and can leave the task stuck in verifying (not blocked), so retry/release/verify all refuse until the row is reset to building.

## 2026-09-16T16:09:51.766Z

Story 8-1 landed DESIGN.md tokens in packages/ui globals.css, but the four live Tauri HTML surfaces stayed native-looking (Settings was an inverted #111 page). The user rejected that look. Visual-system work has to paint the actual WebView HTML, not only the React token file.

## 2026-09-16T18:26:54.954Z

Settings letter-doubling survived a catalog-only fix: bindHandTestLocale never passed a catalog so smash-guards never ran in tauri dev; Skip overlap is input width 100% in a nowrap row; screenshot last-letter doubling is not official en-XA.

