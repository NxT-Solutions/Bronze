# Epic 1 Context: Bootable toolchain and empty app

<!-- Generated from planning artifacts. Regenerate with compile-epic-context if planning docs change. -->

## Goal

A developer can clone the branch and run format/lint/type/unit gates on an empty macOS Tauri workspace with no product behavior and no network.

## Stories

- Story 1.1: Pin toolchain manifests
- Story 1.2: Scaffold pnpm Turbo workspace
- Story 1.3: Scaffold Cargo workspace
- Story 1.4: Scaffold Swift package
- Story 1.5: Empty Tauri app with CSP deny
- Story 1.6: i18n catalogs and pseudo-locales
- Story 1.7: shadcn React Aria foundation
- Story 1.8: Workspace verify aggregate

## Requirements & Constraints

- Production UI must load only packaged assets; strict CSP denies remote origins, eval, frames, objects (SEC-001, ADR-010, ADR-017).
- Tauri capabilities are per-window and least-privilege; WebView may not use shell, arbitrary fs, http, sql, or unrestricted paths (SEC-002, SEC-003, ADR-010).
- Zero runtime network requests, accounts, telemetry, analytics, or hosted services in production-default build (SEC-004, G-05, ADR-001, ADR-017).
- All user-facing strings must come from locale catalog using stable semantic IDs; no hard-coded sentences, no concatenation, build fails on missing keys or placeholder mismatch (I18N-001).
- Locale resolution uses canonical BCP 47 with script-preserving progressive fallback (RFC 4647 style) before falling to English; ICU formatting; per-item content_language stored as BCP 47 or `und` (I18N-002, I18N-003, ADR-014, G-06).
- Core flows must be reachable by keyboard with no timing-dependent gesture required (A11Y-001).
- Semantic roles, accessible names, states, focus order, and live regions are required on controls (A11Y-002).
- UI must support 200% text resize and 400%/320 CSS px reflow without loss of content or function; respect and may strengthen (but not weaken) OS Reduce Motion, Reduce Transparency, Increase Contrast, Differentiate Without Color (A11Y-003, ADR-013).
- Toolchain pins (Node, pnpm, Rust, Swift) must match recorded bootstrap baseline; do not silently raise deployment targets (ADR-002 remains Proposed).
- Reproducible local bootstrap; no secrets, remote fonts, or CDN in manifests or initial packages.
- Planning checks, markdown lint, and gate scripts must pass; verify command aggregates format/lint/types/units/i18n without caching signing paths.
- Proposed ADRs (002, 009, 018) must remain unresolved; do not adopt them.
- No application windows, capture, storage, or product behavior in this epic; crates and packages exist only as empty compile targets.

## Technical Decisions

- Hexagonal ports-and-adapters: WebView (untrusted presentation) → typed IPC → Rust core → (later) versioned C ABI → Swift static library. No JS-to-Swift calls.
- JS: pnpm workspaces + Turborepo; packages for ui (no product state), contracts, i18n, test-support; apps/desktop for Tauri shell.
- Rust: workspace crates (bronze-domain, bronze-capture, bronze-storage, bronze-settings, bronze-diagnostics, bronze-platform, bronze-platform-macos); rustfmt + Clippy `-D warnings`.
- Native: BronzeNative Swift package as static lib (ABI later); no event tap or AX yet.
- Tauri 2: default-deny capabilities for quick/library/settings/onboarding windows; DevTools disabled in release; forbidden-command tests must close.
- i18n: react-i18next + ICU; catalogs for en, en-XA (pseudo), ar-XB (bidi); build-time missing-key enforcement; native strings later share glossary.
- UI foundation: shadcn initialized on React Aria primitives; tokens and CSS hooks for OS prefs; visible focus preserved or equivalently replaced; axe checks on states; semantic HTML preferred over divs.
- Conventions: UTC timestamps, locale-neutral closed enums, opaque IDs, content-free diagnostics, no user strings in code.
- Stack baseline (from bootstrap/arch): Node 24.19, pnpm 11.9 host, Rust 1.98, Swift 6.3.3, Tauri 2, React 19, Biome, Turborepo, Vitest, axe-core; exact crate/lock pins land during scaffolding.
- Development on arm64; macOS min/universal2 deferred (ADR-002 Proposed). No Turbo cache for later TCC/signing.
- Verify gate required before downstream stories; failures must not be weakened to pass.

## UX & Interaction Patterns

- Foundation only: owned React Aria-based components with role/name/focus tests; no clickable divs; focus outlines preserved or replaced.
- CSS hooks for Reduce Motion / Increase Contrast; app overrides strengthen only.
- Pseudo-locale and RTL test paths required in i18n and layout foundations.
- No product UI surfaces or flows yet; composer, panels, etc. come later.

## Cross-Story Dependencies

- 1.2, 1.3, 1.4 depend on 1.1 (pinned package manager and toolchain files).
- 1.5 depends on 1.2 (JS workspace) and 1.3 (Cargo workspace).
- 1.6 depends on 1.2 (i18n package slot).
- 1.7 depends on 1.2 and 1.6 (ui package + catalogs).
- 1.8 depends on 1.2 through 1.7 (full verify aggregate).
- Within-epic ordering must be respected; later epics depend on Epic 1 scaffolding and gates completing.
- All stories block on unresolved Proposed ADRs 002/009/018 per policy; no silent acceptance.
