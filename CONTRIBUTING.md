# Contributing to Castly

Thanks for your interest in contributing! Here's how to get started.

## Development Setup

1. **Prerequisites**: Node.js 20+, Rust 1.75+, an Android phone with USB debugging
2. Clone and install:
   ```bash
   git clone https://github.com/Funsaiki/castly.git
   cd castly
   npm install
   ```
3. Run in dev mode:
   ```bash
   npm run tauri dev
   ```

## Project Architecture

- **`src/`** — React frontend (TypeScript, Tailwind CSS, Zustand)
- **`src-tauri/`** — Rust backend (Tauri v2, Axum, scrcpy protocol, AirPlay)

### Key Files

| File | Purpose |
|------|---------|
| `src-tauri/src/pipeline.rs` | Android mirroring pipeline (scrcpy) |
| `src-tauri/src/airplay/rtsp.rs` | iOS AirPlay RTSP server |
| `src-tauri/src/video/frame_server.rs` | HTTP video/audio streaming |
| `src/lib/mse-player.ts` | WebCodecs video/audio decoder |
| `src/components/viewport/VideoPlayer.tsx` | Video display + input capture |

### How the Video Pipeline Works

```
scrcpy/AirPlay → H.264 frames → Rust HTTP stream → WebCodecs VideoDecoder → Canvas
```

Each frame is sent as raw H.264 Annex-B data with a 4-byte big-endian length prefix over HTTP chunked transfer.

## Making Changes

1. Create a branch: `git checkout -b feature/my-feature`
2. Make your changes
3. Check compilation:
   ```bash
   cd src-tauri && cargo check
   cd .. && npx tsc --noEmit
   ```
4. Test with a real device
5. Commit and push
6. Open a Pull Request

## Areas for Contribution

- **iOS AirPlay testing** — needs testing with real iPhones, FairPlay adjustments
- **Landscape mode** — proper rotation handling
- **macOS/Linux support** — platform-specific adaptations
- **Performance** — latency optimization, memory management
- **UI improvements** — themes, accessibility, responsive design
- **Translations** — add new languages in `src/lib/i18n.tsx`

## Adding a Language

1. Open `src/lib/i18n.tsx`
2. Add a new locale object following the `fr` / `en` pattern
3. Add the locale to the `Locale` type
4. Add a button in the Titlebar language switcher

## Code Style

- Rust: standard `rustfmt` formatting
- TypeScript: Prettier defaults
- Commits: concise, imperative mood ("add feature" not "added feature")

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
