# Epic 2 Context: In-process Swift bridge

<!-- Generated from planning artifacts. Regenerate with compile-epic-context if planning docs change. -->

## Goal

Tauri executable links BronzeNative and completes ABI round trips without JS seeing Swift. This provides the native platform adapter foundation required for reliable capture coordination (CAP-004) and other macOS features while keeping Swift isolated behind a narrow, versioned, memory-safe contract and maintaining a single process identity.

## Stories

- Story 2.1: Versioned C ABI types
- Story 2.2: Rust macOS façade
- Story 2.3: Link static library into Tauri
- Story 2.4: ABI ownership tests

## Requirements & Constraints

- Every capture request is assigned a monotonic ID and reaches exactly one terminal outcome (saved | rejected | failed | cancelled); zero silently dropped requests (CAP-004, G-01, G-02).
- ABI uses fixed-width version, tagged enums, and pointer-plus-length UTF-8; version check fails closed on mismatch; no NUL-terminated string reliance (CAP-004, SEC-002).
- Explicit buffer ownership, one-shot callbacks; no unwind across the FFI boundary (ADR-004).
- Sensitive buffers have no debug descriptions; no user content in diagnostics or logs from the bridge layer (SEC-006, CAP-010).
- bronze-domain and other core crates contain no macOS imports; all access goes through the safe bronze-platform-macos façade (CAP-004).
- Release-like bundle must contain no helper executable; single process identity; startup performs ABI version check (SEC-005, ADR-004, ADR-011).
- FFI boundary contains panics; invalid UTF-8 rejected; double completion forbidden.
- Ownership tests cover empty, embedded NUL, invalid UTF-8, large payloads, cancel, and shutdown-race cases; each completes exactly once or cancels.
- Proposed ADRs (002, 009, 018) remain unresolved; do not adopt.

## Technical Decisions

- Hexagonal ports-adapters: WebView is untrusted presentation; Rust is privileged core (domain, capture coordinator, storage, settings, diagnostics, Tauri commands); Swift is in-process static-library platform adapter only. No JS-to-Swift path; call direction is strictly WebView → IPC → Rust → versioned C ABI → Swift.
- BronzeNative is a static library linked into the Tauri executable; bronze-platform-macos is the only safe Rust façade over the ABI.
- ABI contract invariants: versioned, explicit ownership transfer, one-shot callbacks, fixed-width types, no unwind/exceptions across boundary.
- React never calls Swift; domain never imports macOS frameworks.
- Naming: crates `bronze-*`; native package `BronzeNative`; IDs generated in Rust.
- Single TCC subject for the entire app.
- Diagnostics are content-free and independent of capture payloads.

## Cross-Story Dependencies

- Within epic: Story 2.1 precedes 2.2 (ABI types before façade); 2.2 precedes 2.3 (façade before link); 2.3 precedes 2.4 (linked app before ownership tests).
- Depends on Epic 1: 2.1 on 1.4 (Swift package skeleton); 2.2 on 1.3 (Cargo workspace); 2.3 on 1.5 (Tauri app).
- Enables Epic 3 capture stories (ingress, coordinator, AX provider, event tap) and Epic 5 native shell stories (status item, windows, display prefs).
- All stories block on unresolved Proposed ADRs per policy.
