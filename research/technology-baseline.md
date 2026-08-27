# Dated technology baseline

Registry snapshot: 2026-08-27. This is planning input, not version policy. Re-query security/support and lock exact versions when M0 starts.

| Package/tool | Observed current version |
| --- | ---: |
| `@tauri-apps/cli` | 2.11.4 |
| `@tauri-apps/api` | 2.11.1 |
| React | 19.2.8 |
| TypeScript | 7.0.2 |
| Biome | 2.5.10 |
| Turborepo | 2.10.12 |
| Vite | 8.2.2 |
| shadcn CLI/package | 4.19.0 |
| Tailwind CSS | 4.3.3 |
| Radix Dialog (comparison only) | 1.1.23 |
| i18next | 26.4.0 |
| react-i18next | 17.0.12 |
| Zod | 4.4.3 |
| Vitest | 4.1.11 |
| Playwright | 1.62.1 |
| axe-core | 4.13.0 |
| Testing Library React | 16.3.2 |
| pnpm | 11.24.0 |
| lucide-react | 1.34.0 |
| TanStack Query | 5.102.6 |
| TanStack Virtual | 3.14.10 |

Decisions before install:

- Confirm TypeScript 7 and Vite 8 compatibility across Tauri/shadcn/Biome plugins.
- Prefer supported current patch, but do not combine all newest majors blindly.
- Validate accepted shadcn React Aria base against accessibility and bundle/runtime requirements. Base UI/Radix may inform spike evidence only; do not mix primitive bases in implementation.
- Avoid list virtualization in MVP despite package availability.
- Generate/pin lockfiles and Rust `Cargo.lock`; check Rust MSRV and Xcode/Swift support.
- Record Node/pnpm/Rust/Xcode/macOS minimum in tool-version files.
- Subscribe to Tauri/RustSec/npm advisories and upgrade deliberately.
