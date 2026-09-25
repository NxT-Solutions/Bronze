# bronze-title-model

Local title refine after persist. Persist still writes `compact_title` first.

## Weights

Developer or CI vendors hash-pinned GGUFs. Runtime never fetches.

```
sh crates/bronze-title-model/scripts/vendor-gguf.sh
sh crates/bronze-title-model/scripts/vendor-gguf.sh smol-135
sh crates/bronze-title-model/scripts/vendor-gguf.sh smol-360
sh crates/bronze-title-model/scripts/vendor-gguf.sh qwen-05
sh crates/bronze-title-model/scripts/vendor-gguf.sh all
```

Allow-list (Apache-2.0):

| id | file | SHA-256 | bytes |
| --- | --- | --- | --- |
| smol-135 | `SmolLM2-135M-Instruct-Q4_K_M.gguf` | `2e8040ceae7815abe0dcb3540b9995eaa1fa0d2ca9e797d0a635ae4433c68c2d` | 105454432 |
| smol-360 | `SmolLM2-360M-Instruct-Q4_K_M.gguf` | `2fa3f013dcdd7b99f9b237717fa0b12d75bbb89984cc1274be1471a465bac9c2` | 270590880 |
| qwen-05 | `qwen2.5-0.5b-instruct-q4_k_m.gguf` | `74a4da8c9fdbcd15bd1f6d01d621410d31c6fc00986f5eb687824e7b93d7a9db` | 491400032 |

git-lfs is not required. GGUF files are gitignored. Capture and runtime never fetch.

`vendor/MANIFEST`, `vendor/manifests/*`, `vendor/LICENSE`, and `vendor/NOTICE` record each pin.

Load order per active file: `set_weights_path` when that filename matches, else `BRONZE_TITLE_WEIGHTS` when that filename matches, else `set_weights_dir`, else `vendor/` next to this crate, else workspace walk, else `models/` beside the executable, else `Resources/models`. No WebView path.

## Runtime

`llama-cpp-2` 0.1.156, local file only, CPU (`n_gpu_layers = 0`). Settings `general.titleModel` selects `extractive`, `smol-135`, `smol-360`, or `qwen-05`. A change unloads the previous context and loads the new file on thread `bronze-title-model`. Capture is not blocked.

First load may use up to 45 s. Generate after load stays 8 s. Hash is checked once per path and cached.

## Diagnostics

Local stderr only. Lines start with `bronze-title:` and never include captured text.

Stages: `weights resolved source={bundle|override|env|vendor|workspace|exe|resources}`, `hash ok`, `model loaded`, `switch scheduled tier=`, `refine scheduled chars=N`, `refine attempted`, `refine generated`, `refine applied`. Those load stages also update `title_engine_status` / the `title-engine-status` event (`idle`, `loading`, `hashing`, `ready`, `missing`, `failed`).

Fallback reasons: `missing_weights`, `bad_hash`, `unreadable`, `timeout`, `short_body`, `ungrounded`, `first_sentence`, `empty`, `stale_body`, `extractive`. After a generate attempt that falls back, stderr also logs `fallback raw_len=N raw_preview=` with the first 40 characters of model output (controls stripped, never the capture body). `empty` means no usable cleaned text. `first_sentence` is a v4 opening-echo reject. `ungrounded` means no shared 3+ character term.

In `tauri dev`, grep the cargo/tauri terminal for `bronze-title:`.

## Fallback

Missing file, hash mismatch, timeout, empty output, first-sentence echo, load failure, extractive, a body already at most 40 characters, or a title that shares no 3+ character term with the body returns no refine. The stored `compact_title` stays. A leaked `Title:` prefix is stripped before clamp.
