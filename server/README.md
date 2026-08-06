# Legacy Go Server (Migrated to Rust)

> ⚠️ **Notice**: The backend server for NanoKVM has been completely migrated from Go to pure Rust in `crates/nanokvm-server`.
> 
> See the main repository [README.md](../README.md) for building and deploying `RustyNanoKVM`.

## Shared Objects (`dl_lib`)

This directory contains `dl_lib/libkvm.so` and related vendor dynamic libraries used by `nanokvm-vision` for hardware-accelerated video capture on the SG2002 RISC-V SoC.
