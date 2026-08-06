# 🦀 RustyNanoKVM

[![Rust](https://img.shields.io/badge/Rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-RISC--V%20SG2002-green.svg)](https://wiki.sipeed.com/hardware/en/kvm/NanoKVM/introduction.html)

A high-performance, lightweight **pure Rust rewrite** of the NanoKVM IP-KVM software stack for RISC-V devices (Sipeed LicheeRV Nano / SG2002).

`RustyNanoKVM` replaces the heavy legacy daemon stack with a modular, memory-efficient Rust server (~2 MB binary) that interfaces directly with the Sophgo multimedia ISP driver for hardware-accelerated 1080p video capture, low-latency MJPEG streaming, USB HID emulation, and board management.

---

## 🌟 Key Features

* **⚡ Hardware-Accelerated Video Capture**: Direct FFI integration with the SG2002 video processing subsystem for 1080p@60fps HDMI input capture.
* **🎥 Real-Time MJPEG & Snapshot Streaming**: Standard HTTP multipart (`multipart/x-mixed-replace`) streaming and single-frame 1080p JPEG snapshots.
* **⌨️ USB HID Emulation**: Full keyboard and relative/absolute mouse control via Linux USB Gadget (`/dev/hidg*`).
* **🔌 Board Hardware Control**: GPIO, ATX power button toggling, I2C communication, and SSD1306 OLED display management.
* **🪶 Ultra-Low Memory & CPU Footprint**: ~2 MB compiled binary size using pure async Rust (Axum & Tokio), using a fraction of the system resources.

---

## 📂 Workspace Architecture

The project is structured as a modular Cargo workspace:

```
crates/
├── nanokvm-server/   # Main Axum HTTP & WebSocket API server
├── nanokvm-vision/   # Hardware video capture (FFI bindings & MJPEG/H.264 stream logic)
├── nanokvm-hid/      # USB Gadget HID keyboard, mouse, and keycode translation
├── board-support/    # Hardware peripherals (GPIO, ATX power, I2C, OLED display, QR codes)
└── nanokvm-core/     # Configuration loader, errors, and common types
```

---

## 🛠️ Building & Testing

### Prerequisites

- Rust 1.85+ (`rustup target add riscv64gc-unknown-linux-musl`)
- Cross-compiler toolchain for RISC-V: `riscv64-linux-gnu-gcc`

### Local Development & Unit Tests

Run workspace unit tests using mock hardware mode:

```bash
cargo test --workspace --features mock
```

### Cross-Compiling for NanoKVM (SG2002 RISC-V)

To build the release binary for the target RISC-V hardware:

```bash
CC_riscv64gc_unknown_linux_musl=riscv64-linux-gnu-gcc \
cargo build --target riscv64gc-unknown-linux-musl --release
```

The compiled binary will be generated at:
`target/riscv64gc-unknown-linux-musl/release/nanokvm-server`

---

## 🚀 Deployment to NanoKVM Device

1. **Transfer Binary & Web Frontend**:

   ```bash
   # Copy the compiled binary
   scp target/riscv64gc-unknown-linux-musl/release/nanokvm-server root@<nanokvm-ip>:/tmp/server/nanokvm-server
   
   # Copy static web assets
   scp -r web root@<nanokvm-ip>:/tmp/server/
   ```

2. **Run Server on Device**:

   ```bash
   ssh root@<nanokvm-ip>
   LD_LIBRARY_PATH=/tmp/server/dl_lib /lib/ld-musl-riscv64v0p7_xthead.so.1 /tmp/server/nanokvm-server
   ```

---

## 📡 API Reference

| Endpoint | Method | Description |
| :--- | :--- | :--- |
| `/api/stream/mjpeg` | `GET` | Continuous HTTP MJPEG stream (`multipart/x-mixed-replace`) |
| `/api/stream/snapshot` | `GET` | Single 1080p JPEG snapshot image (`image/jpeg`) |
| `/api/application/info` | `GET` | System and server metadata |
| `/api/application/version` | `GET` | Server version |
| `/api/hid/keyboard` | `POST` | Send keyboard input events |
| `/api/hid/mouse` | `POST` | Send mouse movement and button events |
| `/api/vm/power/short` | `POST` | Trigger ATX power button short press |
| `/api/vm/power/long` | `POST` | Trigger ATX power button long press (force off) |
| `/api/vm/reset` | `POST` | Trigger ATX reset button press |

---

## 📜 License

Distributed under the [MIT License](LICENSE).
