# bronze-title-model

Local title refine after persist. Persist still writes `compact_title` first.

## Weights

Developer or CI vendors once:

```
sh bronze-title-model/scripts/vendor-gguf.sh
```

The script downloads `SmolLM2-360M-Instruct-Q4_K_M.gguf` from the pinned Hugging Face revision and checks SHA-256 `2fa3f013dcdd7b99f9b237717fa0b12d75bbb89984cc1274be1471a465bac9c2`. git-lfs is not required. The GGUF is gitignored. Capture and runtime never fetch.

`vendor/MANIFEST`, `vendor/LICENSE`, and `vendor/NOTICE` record HuggingFaceTB/SmolLM2-360M-Instruct (Apache-2.0), the bartowski Q4_K_M file, revision, and hash.

Load order: `set_weights_path` (Rust setup only, when that file exists), else `BRONZE_TITLE_WEIGHTS`, else `vendor/` next to this crate (`CARGO_MANIFEST_DIR` of `bronze-title-model`), else workspace walk from the crate, cwd, or executable looking for `bronze-title-model/vendor/`, else `models/` beside the executable, else `Resources/models`. A missing override does not hide later candidates. No WebView path.

## Runtime

`llama-cpp-2` 0.1.156, local file only, CPU (`n_gpu_layers = 0`). The crate does not enable hub download. On Apple Silicon the crate still compiles Metal support; this worker does not offload and does not add JIT or unsigned-executable-memory entitlements.

First load may use up to 45 s on thread `bronze-title-model`. Generate after load stays 8 s. Capture is not blocked. Hash is checked once and cached.

## Diagnostics

Local stderr only. Lines start with `bronze-title:` and never include captured text.

Stages: `weights resolved source={bundle|override|env|vendor|workspace|exe|resources}`, `hash ok`, `model loaded`, `refine scheduled chars=N`, `refine attempted`, `refine generated`, `refine applied`.

Fallback reasons: `missing_weights`, `bad_hash`, `unreadable`, `timeout`, `short_body`, `ungrounded`, `empty`, `stale_body`.

In `tauri dev`, grep the cargo/tauri terminal for `bronze-title:`. Optional: `BRONZE_TITLE_WEIGHTS=/absolute/path/to/SmolLM2-360M-Instruct-Q4_K_M.gguf`.

## Fallback

Missing file, hash mismatch, timeout, empty output, load failure, a body already at most 40 characters, or a title that shares no 3+ character term with the body returns no refine. The stored `compact_title` stays. Short input can still produce unrelated titles; those are dropped rather than stored.
