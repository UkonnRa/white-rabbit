# AGENTS.md — endpoint-tauri

## Known Issues

### Tauri MockRuntime tests require a manifest workaround on Windows

Tests using `tauri::test::MockRuntime` or `WebviewWindowBuilder` crash on
Windows with `STATUS_ENTRYPOINT_NOT_FOUND` (0xc0000139) without a local fix.

- **Root cause and fix:** see comments in `endpoint-tauri/build.rs`.
- **Upstream:** [tauri#13419](https://github.com/tauri-apps/tauri/issues/13419), [tauri#14580](https://github.com/tauri-apps/tauri/issues/14580).
- **New tests** need no extra config — the `build.rs` fix applies to all test targets automatically.
- **Removal:** once upstream fixes land, delete `tests/comctl32-v6.exe.manifest`, strip the `#[cfg(windows)]` block from `build.rs`, and verify on Windows.
