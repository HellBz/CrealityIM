# CrealityIM

An unofficial native desktop chat client for **Creality Cloud** messaging, built with [Tauri](https://tauri.app/) and Rust.

![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-blue)
![License](https://img.shields.io/badge/license-MIT-green)
![Version](https://img.shields.io/github/v/release/HellBz/CrealityIM)

## Screenshots

<table>
  <tr>
    <td><img src="screenshot/login.png" width="480" alt="Login"/></td>
    <td><img src="screenshot/main.png" width="480" alt="Main"/></td>
  </tr>
  <tr>
    <td colspan="2" align="center"><img src="screenshot/chat.png" width="480" alt="Chat"/></td>
  </tr>
  <tr>
    <td><img src="screenshot/user-search.png" width="480" alt="User Search"/></td>
    <td><img src="screenshot/model-search.png" width="480" alt="Model Search"/></td>
  </tr>
</table>

## Features

- **OAuth Login** via Creality Cloud account (id.creality.com)
- **Real-time messaging** over WebSocket (Tencent IM)
- **File & image sharing** with file-type icons and in-app preview
- **3D model sharing** directly from your Creality Cloud library
- **User search** on Creality Cloud to start new chats
- **Native notifications** for new messages (Windows, macOS, Linux)
- **Contact list** with avatars, unread badges and last message preview
- **Message recall** and local delete
- **Dark mode** UI with Creality green accent
- **Auto-reconnect** with token refresh
- **Secure credential storage** via OS keychain

## Download

Pre-built installers are available on the [Releases](https://github.com/HellBz/CrealityIM/releases) page:

| Platform | Format |
|---|---|
| Windows | `.exe` (NSIS Installer) / `.msi` |
| macOS | `.dmg` |
| Linux | `.deb` / `.AppImage` |

## Build from Source

### Prerequisites

- [Node.js](https://nodejs.org/) 20+
- [Rust](https://rustup.rs/) (stable)
- On Linux: `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf`

### Steps

```bash
git clone https://github.com/HellBz/CrealityIM.git
cd CrealityIM
npm install
npm run tauri build
```

The installer will be at:
- Windows: `src-tauri/target/release/bundle/nsis/`
- macOS: `src-tauri/target/release/bundle/dmg/`
- Linux: `src-tauri/target/release/bundle/deb/` or `appimage/`

### Development

```bash
npm run tauri dev
```

## Tech Stack

- **Frontend**: Vanilla JS / HTML / CSS (no framework)
- **Backend**: Rust + Tauri v2
- **Messaging**: Tencent IM via WebSocket
- **Auth**: Creality Cloud OAuth

## Notes

- Credentials are stored locally using the OS keychain via Tauri's secure storage.
- This is an unofficial client — not affiliated with Creality.

## License

MIT
