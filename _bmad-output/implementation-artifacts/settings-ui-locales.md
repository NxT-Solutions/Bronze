# Settings UI locales

Status: done

## Intent

Settings → General persists `SettingsV1.general.locale` for **en** (default), **nl**, **fr**, **de**, **es**, and **it**. WebView chrome applies the matching catalog, sets `html lang`/`dir`, labels language options with endonyms and option `lang`, and keeps item bodies `lang="und" dir="auto"`. `system` and unknown tags map to **en**. Native menus follow the persisted locale at launch. No public human linguistic QA claim.

**Requirements:** I18N-001, I18N-002, I18N-003, G-06, SET-001
**ADRs:** ADR-014 accepted. Proposed ADR-002 / ADR-009 / ADR-018 remain unresolved.

## Overview

Allowed persist tags are `system`, `en`, `nl`, `fr`, `de`, `es`, `it`, `en-XA`, and `ar-XB`. The Settings switcher offers the six human tags as endonyms. Effective UI locale maps `system` and unknown values to **en**, and a shipped primary subtag (`nl-BE`) to that catalog.

`apply-locale` loads `ui_catalog`, sets document language and direction, and fills `[data-i18n]` chrome (plus placeholder / title / aria-label). Item title and body slots are skipped so captured text is not relabeled. Queue and Library cards stay `lang="und" dir="auto"`.

Advertised locales remain **en**, **en-XA**, and **ar-XB**. Shipped nl/fr/de/es/it catalogs are not a public locale claim. A live switch broadcasts `ui-locale-changed` to open WebViews. Native app and status menus resolve the same glossary at launch and do not rebuild on that event.

## File list

- `bronze-settings/src/schema.rs` — `PERSISTED_LOCALE_TAGS`, `general.locale` default `system`
- `apps/desktop/src-tauri/src/catalog.rs` — `SHIPPED_UI_LOCALES`, `ADVERTISED_LOCALES`, `html_lang`
- `apps/desktop/src-tauri/src/live_session.rs` — `effective_ui_locale`, `ui_catalog`
- `apps/desktop/src/settings.html` — General language `<select>` with endonyms and option `lang`
- `apps/desktop/src/settings-live.mjs` — persist / reset / `emitUiLocaleChanged`
- `apps/desktop/src/apply-locale.mjs` — catalog apply, `html lang`/`dir`
- `packages/i18n/locales/{en,nl,fr,de,es,it,en-XA,ar-XB}/app.json`
- `README.md` — Settings language line and advertised-vs-shipped limit
- `docs/11-i18n-localization.md` — locale model, catalogs, rollout
- `docs/12-settings-and-shortcuts.md` — schema, default, tests

## Notes

- Human linguistic QA remains backlog (stories 3.9, 3.10, 9.3).
- I18N-001 through I18N-004 and G-06 stay planned in `docs/15`.
- Live language switch does not rebuild the native app menu.
