# Castly

> Universal phone screen mirroring app — Android + iOS to PC.

Castly mirrors your phone screen on your computer with full control, audio forwarding, and recording capabilities. Works with Android (via scrcpy) and iOS (via AirPlay).

![Castly Screenshot](https://img.shields.io/badge/platform-Windows-blue) ![License](https://img.shields.io/badge/license-MIT-green) ![Built with](https://img.shields.io/badge/built%20with-Rust%20%2B%20React-orange)

## Features

### Android
- **USB & Wi-Fi mirroring** via scrcpy v3.1
- **Full control** — touch, keyboard, scroll, back/home/recent buttons
- **Audio forwarding** — hear your phone on your PC (Opus codec)
- **Screen off mode** — turn off the phone screen to save battery while mirroring
- **Quality presets** — Eco (720p/30fps), Balanced (1280p/30fps), Quality (1920p/60fps)
- **Wireless debugging** — pair via 6-digit code (Android 11+)

### iOS
- **AirPlay receiver** — your PC appears as an AirPlay target
- **Zero setup on iPhone** — just tap Screen Mirroring from Control Center
- **Display only** — no touch control (AirPlay limitation)

### General
- **Video recording** — save mirroring sessions as WebM
- **Screenshots** — capture the current frame as WebP
- **Bilingual** — French & English interface
- **Dark theme** — modern UI with smooth animations
- **Low latency** — WebCodecs API for hardware-accelerated H.264 decoding

## Tech Stack

| Component | Technology |
|-----------|-----------|
| Desktop framework | [Tauri v2](https://tauri.app/) (Rust) |
| Frontend | React 19 + TypeScript |
| Styling | Tailwind CSS 4 |
| State management | Zustand |
| Video decoding | WebCodecs API (hardware-accelerated) |
| Audio decoding | WebCodecs AudioDecoder (Opus / AAC) |
| Android protocol | scrcpy v3.1 (H.264 over ADB) |
| iOS protocol | AirPlay (mDNS + RTSP + H.264) |
| HTTP streaming | Axum (Rust) |

## Architecture

```
Phone (Android/iOS)
  |
  |-- Android: scrcpy H.264 + Opus via ADB (USB/Wi-Fi)
  |-- iOS: AirPlay H.264 + AAC-ELD via mDNS/RTSP
  |
  v
Rust Backend (Tauri)
  |-- ADB client / AirPlay RTSP server
  |-- Frame server (axum HTTP on localhost)
  |
  v
React Frontend
  |-- WebCodecs VideoDecoder -> Canvas rendering
  |-- WebCodecs AudioDecoder -> AudioContext playback
  |-- Touch/keyboard events -> scrcpy control protocol
```

## Getting Started

### Prerequisites

- **Windows 10/11** (macOS/Linux support planned)
- **Node.js** 20+
- **Rust** 1.75+
- **Android phone** with USB debugging enabled, OR
- **iPhone/iPad** on the same Wi-Fi network

### Install & Run

```bash
# Clone the repo
git clone https://github.com/Funsaiki/castly.git
cd castly

# Install dependencies
npm install

# Run in development mode
npm run tauri dev
```

### Connect Your Phone

#### Android (USB)
1. Enable **Developer Options** (tap Build Number 7 times in Settings > About)
2. Enable **USB Debugging** in Developer Options
3. Plug your phone via USB
4. Allow USB debugging when prompted on the phone
5. Double-click the device in Castly to start mirroring

#### Android (Wi-Fi)
1. Enable **Wireless Debugging** in Developer Options
2. Tap **Pair device with pairing code**
3. Enter the IP, pairing port, code, and connection port in Castly's Wi-Fi section
4. Click **Pair**

#### iOS (AirPlay)
1. Ensure PC and iPhone are on the same Wi-Fi
2. Open **Control Center** on iPhone
3. Tap **Screen Mirroring**
4. Select **Phone Mirror**

## Controls

| Action | Input |
|--------|-------|
| Tap | Click |
| Swipe | Click + drag |
| Scroll | Mouse wheel |
| Type | Keyboard (click video area first for focus) |
| Back | `Escape` key or Back button |
| Home | `Home` key or Home button |
| Recent apps | Recent button |

## Building

```bash
# Build for production
npm run tauri build
```

The installer will be generated in `src-tauri/target/release/bundle/`.

## Project Structure

```
castly/
├── src/                    # React frontend
│   ├── components/         # UI components (Sidebar, Titlebar, VideoPlayer...)
│   ├── lib/                # Core logic (WebCodecs player, Tauri commands, i18n)
│   ├── stores/             # Zustand state (devices, sessions, settings)
│   └── styles/             # Tailwind CSS globals & animations
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── adb/            # ADB client, scrcpy protocol
│   │   ├── airplay/        # AirPlay mDNS, RTSP server, receiver
│   │   ├── commands/       # Tauri IPC command handlers
│   │   ├── discovery/      # Device scanning (ADB + mDNS)
│   │   ├── video/          # Frame server (HTTP streaming)
│   │   ├── pipeline.rs     # Mirror pipeline orchestration
│   │   └── state.rs        # App state management
│   └── resources/          # Bundled scrcpy-server.jar
└── public/                 # Static assets (help images)
```

## Known Limitations

- **iOS control**: AirPlay is display-only — no touch/keyboard injection
- **iOS FairPlay**: DRM-protected content (Netflix, YouTube) won't stream via AirPlay
- **Landscape mode**: Rotation detection is basic, may need manual reconnection
- **Platform**: Windows only for now (macOS/Linux planned)

## Credits

- [scrcpy](https://github.com/Genymobile/scrcpy) — Android screen mirroring protocol
- [Tauri](https://tauri.app/) — Desktop app framework
- [UxPlay](https://github.com/antimof/UxPlay) — AirPlay protocol reference

## License

[MIT](LICENSE)
