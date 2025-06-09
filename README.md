# TulSU PE Timetable

A convenient app for TulSU PE timetable management.

## Project setup

1. Install rust (preferably via rustup), Node.js and Tauri.
2. Enable corepack: `corepack enable`.
3. Install lld (for faster builds). See `src-tauri/.cargo/config.toml` for instructions.
4. Run `pnpm install`.
5. Run `tauri dev`

## Building

```
pnpm tauri build 
pnpm tauri bundle
```
