# Secure Passage

**Navigating the Digital Seas with Privacy and Security**

[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Nym](https://img.shields.io/badge/network-Nym%20mixnet-purple.svg)](https://nym.com/)

Secure Passage is a desktop privacy app for **encrypted chat** and **file sharing** over the [Nym mixnet](https://nym.com/). It is a Rust rewrite of the original [Python / Tor Secure Passage app](https://github.com/SampleBias/secure-passage-app), keeping the familiar dark mono UI while replacing Tor hidden services with the official **Nym Rust SDK** (`nym-sdk`).

Traffic is Sphinx-encrypted and routed through mix nodes for network-level privacy. Application payloads are additionally protected with **AES-256-GCM** session keys that you share out-of-band (copy / QR).

---

## Table of contents

- [Features](#features)
- [Tor → Nym](#tor--nym)
- [Requirements](#requirements)
- [Install & run](#install--run)
- [Usage](#usage)
- [Architecture](#architecture)
- [Security notes](#security-notes)
- [Project layout](#project-layout)
- [Roadmap](#roadmap)
- [Contributing](#contributing)
- [License](#license)

---

## Features

### Included (MVP)

| Feature | Description |
|---|---|
| **Nym connect** | Embedded mixnet client with persistent identity |
| **Secure Chat** | Host / join encrypted messaging over `MixnetStream` |
| **File Sharing** | Chunked encrypted transfer, optional password, auto-shutdown |
| **Logs** | In-app log window with filter |
| **Kill** | Tear down the mixnet client and return to the connect screen |

### UI

- Dark theme (`#333333` / `#3A3A3A`) with purple accent (`#A68CFF`)
- Monospace typography (IBM Plex Mono–style look)
- Ship banner artwork carried over from the Python app
- Navigation: File Sharing · Website Hosting · Secure Chat · Mixnet Browser  
  *(Hosting and Browser are placeholders for a later phase)*

### Deferred

- **Website Hosting** — anonymous static hosting via the mixnet
- **Mixnet Browser** — SOCKS / IPR browsing with an embedded webview

---

## Tor → Nym

| Python / Tor app | This project (Rust / Nym) |
|---|---|
| `.onion` hidden service addresses | Nym `Recipient` addresses (`identity.encryption@gateway`) |
| External Tor + Stem control | Embedded `nym-sdk` `MixnetClient` |
| Flask HTTP behind a hidden service | Direct length-prefixed protocols on `MixnetStream` |
| Fernet (AES-128-CBC + HMAC) | AES-256-GCM (new key format; **not** wire-compatible with the Python app) |
| PyQt5 | egui / eframe |

**Expect higher or variable latency** (mix delays and cover traffic). For stream-based chat and file transfer, **both peers should be online**.

---

## Requirements

- **Rust** 1.75+ (developed against 1.97)
- A desktop environment supported by [egui](https://github.com/emilk/egui) (Linux tested)
- Network access to the **Nym mainnet** mixnet
- ~disk space for a debug build of `nym-sdk` and dependencies (release builds are smaller)

---

## Install & run

### From source

```bash
git clone https://github.com/SampleBias/Secure-Passage.git
cd Secure-Passage
cargo run --release -p secure-passage-ui
```

The binary is named `secure-passage`.

### Tests

```bash
cargo test -p secure-passage-core
```

### Persistent identity

On first successful connect, keys are stored under:

```text
~/.local/share/secure-passage/nym/
```

Your Nym address stays stable across restarts until you delete that directory.

---

## Usage

### Startup flow

1. Launch the app  
2. **Connect to Nym** — wait for gateway bootstrap (can take a minute)  
3. **Enter App**

### Secure Chat

**Host**

1. Open **Secure Chat** → Host  
2. Copy or show the QR for your **Nym address** + **session key**  
3. Click **Start Hosting** and wait for a peer  

**Join**

1. Paste the host address and session key  
2. Click **Join Chat**  
3. Send messages — payloads are AES-GCM encrypted before mixnet transit  

Share address and key through a **separate secure channel**.

### File Sharing

**Send**

1. Choose a file, note address + session key (optional password)  
2. **Start Sharing** and wait for the receiver  

**Receive**

1. Enter host address + session key (and password if required)  
2. Pick a download folder → **Receive File**  
3. Transfer verifies a SHA-256 checksum after decrypt  

---

## Architecture

```text
┌─────────────────────────────────────────────────────────┐
│  secure-passage-ui (egui)                               │
│    Nym connect · chat · files · logs · status           │
└───────────────────────────┬─────────────────────────────┘
                            │ commands / events
┌───────────────────────────▼─────────────────────────────┐
│  secure-passage-core                                    │
│    NymService (MixnetClient + MixnetStream)             │
│    SessionKey (AES-256-GCM)                             │
│    Chat + file framing protocols                        │
└───────────────────────────┬─────────────────────────────┘
                            │
                    Nym mixnet (Sphinx)
```

- **Message framing**: `[u32 BE length][payload]` over `AsyncRead` / `AsyncWrite` streams  
- **Chat**: encrypted JSON handshake + messages (`SPCHAT01`)  
- **Files**: encrypted metadata (`SPFILE01`) + chunked ciphertext with end-of-stream marker  

Primary dependency: [`nym-sdk`](https://docs.rs/nym-sdk/) — see also [Nym Rust developer docs](https://nym.com/docs/developers/rust).

---

## Security notes

**Strengths**

- Network-level privacy via the Nym mixnet (Sphinx multi-hop)  
- App-layer AES-256-GCM on chat and file payloads  
- No central application server for MVP chat/files  
- Open source (MIT) — auditable  

**Limitations**

- Session keys are only as safe as the channel you use to share them  
- Streams need live peers; gateways help with messaging but streams are interactive  
- Endpoint compromise (malware, screen capture) defeats client-side encryption  
- Not compatible with sessions from the Python/Tor app  

This software is provided as-is. Use it with operational security in mind.

---

## Project layout

```text
Secure-Passage/
├── Cargo.toml                 # workspace
├── LICENSE                    # MIT
├── README.md
├── assets/images/             # banner SVG + app icons
├── crates/
│   ├── secure-passage-core/   # nym, crypto, protocols (no GUI)
│   └── secure-passage-ui/     # egui desktop binary
└── .cargo/config.toml
```

---

## Roadmap

- [x] Rust + egui shell matching original layout  
- [x] Nym connect with persistent identity  
- [x] Encrypted chat over MixnetStream  
- [x] Encrypted file transfer  
- [ ] Website hosting via mixnet  
- [ ] Mixnet browser (SOCKS / IPR + webview)  
- [ ] Packaging (AppImage / Flatpak / installers)  

---

## Contributing

Issues and pull requests are welcome. For large changes, open an issue first so we can align on design (especially protocol and Nym integration details).

```bash
cargo fmt
cargo test -p secure-passage-core
cargo build -p secure-passage-ui
```

---

## License

This project is licensed under the [MIT License](LICENSE).

---

## Acknowledgments

- Original Python/Tor app: [SampleBias/secure-passage-app](https://github.com/SampleBias/secure-passage-app)  
- [Nym](https://nym.com/) mixnet and [`nym-sdk`](https://crates.io/crates/nym-sdk)  
- [egui](https://github.com/emilk/egui) for the desktop UI  
