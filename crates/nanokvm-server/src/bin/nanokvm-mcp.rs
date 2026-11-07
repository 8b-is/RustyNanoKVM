#![forbid(unsafe_code)]
//! `nanokvm-mcp` — Model Context Protocol (MCP) Server & CLI for `RustyNanoKVM` (`rustykvm` / `ayekvm`).
//!
//! Connects to a live `RustyNanoKVM` appliance over HTTP (`rustykvm:80` / `ayekvm:80`), captures the
//! 1080p HDMI frame + audio state, runs the **Balanced Ternary (`{-1, 0, +1}`, Base-243 29-byte)**
//! Video Capture Model, **PhoeniX `.8a∞` (`0x38 0x61 0xE2 0x9E`)** Audio Context Model, and
//! **Smart-Tree × MemNet (`MemnetCapsule64` `0x4D4E` + `WaveFrame79` `MEM|8` v2 + file tree/read/search)**,
//! serving real-time structural and filesystem context to any MCP-compatible AI agent over JSON-RPC 2.0 (`stdio`).
//!
//! Strict 8b-is No-Float Law: 100% integer, balanced ternary `{-1, 0, +1}`, and reduced `p/q` rationals.

use std::env;
use std::fs;
use std::io::{BufRead, Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::path::Path;
use std::time::Duration;

use nanokvm_core::memnet::{
    MemnetAddress, MemnetCapsule64, WaveFrame79, broadcast_memnet_capsule_udp,
    read_memnet_file_chunk, scan_smart_tree_directory, search_smart_tree_files,
};
use nanokvm_vision::{TernaryMcpContextPacket, TernaryVideoAnalyzer};
use serde_json::{Value, json};

/// Default `RustyNanoKVM` (`rustykvm` / `ayekvm`) hostname:port on the local mesh.
pub const DEFAULT_KVM_ADDR: &str = "rustykvm:80";

/// Look up a `HostName` mapping for `alias` inside `$HOME/.ssh/config` if DNS / `/etc/hosts`
/// does not define the bare hostname directly.
fn resolve_ssh_config_hostname(alias: &str) -> Option<String> {
    let home = env::var("HOME").ok()?;
    let cfg = fs::read_to_string(Path::new(&home).join(".ssh/config")).ok()?;
    let mut in_matching_host = false;
    for raw_line in cfg.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut parts = line.split_whitespace();
        let Some(keyword) = parts.next() else {
            continue;
        };
        if keyword.eq_ignore_ascii_case("Host") {
            in_matching_host = parts.any(|pattern| pattern.eq_ignore_ascii_case(alias));
        } else if in_matching_host && keyword.eq_ignore_ascii_case("HostName") {
            if let Some(target) = parts.next() {
                return Some(target.to_string());
            }
        }
    }
    None
}

/// Resolve a `host` or `host:port` string to a `SocketAddr` via DNS, `/etc/hosts`, `~/.ssh/config`, or `.local` mDNS.
pub fn resolve_kvm_socket_addr(addr_str: &str) -> Result<SocketAddr, String> {
    let trimmed = addr_str.trim();
    let (host, port) = match trimmed.rsplit_once(':') {
        Some((h, p)) => (h, p.parse::<u16>().unwrap_or(80)),
        None => (trimmed, 80u16),
    };

    if let Ok(mut iter) = (host, port).to_socket_addrs() {
        if let Some(addr) = iter.next() {
            return Ok(addr);
        }
    }

    if let Some(ssh_target) = resolve_ssh_config_hostname(host) {
        if let Ok(mut iter) = (ssh_target.as_str(), port).to_socket_addrs() {
            if let Some(addr) = iter.next() {
                return Ok(addr);
            }
        }
    }

    if !host.ends_with(".local") {
        let mdns_host = format!("{host}.local");
        if let Ok(mut iter) = (mdns_host.as_str(), port).to_socket_addrs() {
            if let Some(addr) = iter.next() {
                return Ok(addr);
            }
        }
    }

    Err(format!("unable to resolve host '{addr_str}' via DNS, /etc/hosts, ~/.ssh/config, or mDNS"))
}

/// Decode an HTTP `Transfer-Encoding: chunked` body into contiguous raw bytes.
fn decode_chunked_body(body: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(body.len());
    let mut idx = 0usize;
    while idx < body.len() {
        let Some(rel_end) = body[idx..].windows(2).position(|w| w == b"\r\n") else {
            break;
        };
        let len_line = &body[idx..idx + rel_end];
        let hex_str = core::str::from_utf8(len_line)
            .unwrap_or("")
            .split(';')
            .next()
            .unwrap_or("")
            .trim();
        let Ok(chunk_len) = usize::from_str_radix(hex_str, 16) else {
            break;
        };
        idx += rel_end + 2;
        if chunk_len == 0 || idx + chunk_len > body.len() {
            break;
        }
        out.extend_from_slice(&body[idx..idx + chunk_len]);
        idx += chunk_len;
        if idx + 2 <= body.len() && &body[idx..idx + 2] == b"\r\n" {
            idx += 2;
        }
    }
    if out.is_empty() {
        body.to_vec()
    } else {
        out
    }
}

/// Perform a minimal zero-float HTTP/1.1 GET request and return the raw response body bytes.
pub fn http_get_bytes(addr_str: &str, path: &str) -> Result<Vec<u8>, String> {
    let addr = resolve_kvm_socket_addr(addr_str)?;
    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_millis(2500))
        .map_err(|e| format!("connect {addr_str} ({addr}) failed: {e}"))?;
    let _ = stream.set_read_timeout(Some(Duration::from_millis(3000)));
    let _ = stream.set_write_timeout(Some(Duration::from_millis(2000)));

    let req = format!(
        "GET {path} HTTP/1.1\r\nHost: {addr_str}\r\nConnection: close\r\n\r\n"
    );
    stream
        .write_all(req.as_bytes())
        .map_err(|e| format!("write failed: {e}"))?;

    let mut raw = Vec::with_capacity(131_072);
    stream
        .read_to_end(&mut raw)
        .map_err(|e| format!("read failed: {e}"))?;

    if let Some(pos) = raw.windows(4).position(|w| w == b"\r\n\r\n") {
        let hdr_str = String::from_utf8_lossy(&raw[..pos]).to_ascii_lowercase();
        let body = &raw[pos + 4..];
        if hdr_str.contains("transfer-encoding: chunked") {
            Ok(decode_chunked_body(body))
        } else {
            Ok(body.to_vec())
        }
    } else {
        Err("missing HTTP header separator".to_string())
    }
}

/// Perform a minimal HTTP/1.1 POST request with a JSON body.
pub fn http_post_json(addr_str: &str, path: &str, body_json: &str) -> Result<String, String> {
    let addr = resolve_kvm_socket_addr(addr_str)?;
    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_millis(2500))
        .map_err(|e| format!("connect {addr_str} ({addr}) failed: {e}"))?;
    let _ = stream.set_read_timeout(Some(Duration::from_millis(2500)));
    let _ = stream.set_write_timeout(Some(Duration::from_millis(2000)));

    let req = format!(
        "POST {path} HTTP/1.1\r\nHost: {addr_str}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body_json.len(),
        body_json
    );
    stream
        .write_all(req.as_bytes())
        .map_err(|e| format!("write failed: {e}"))?;

    let mut raw = Vec::new();
    stream
        .read_to_end(&mut raw)
        .map_err(|e| format!("read failed: {e}"))?;

    if let Some(pos) = raw.windows(4).position(|w| w == b"\r\n\r\n") {
        Ok(String::from_utf8_lossy(&raw[pos + 4..]).to_string())
    } else {
        Ok(String::from_utf8_lossy(&raw).to_string())
    }
}

/// Build the 64-byte `MemnetCapsule64` (`0x4D4E`) and 79-byte `WaveFrame79` (`MEM|8` v2)
/// fused with a live `TernaryMcpContextPacket`.
pub fn build_memnet_context(packet: &TernaryMcpContextPacket) -> Value {
    let mut cap = MemnetCapsule64::default();
    cap.seq = (packet.frame_seq & 0xFFFF_FFFF) as u32;
    cap.node_trit = if packet.active_glyph_cells > 0 { 1 } else { 0 };
    cap.wave_b3 = packet
        .spatial_base243_hex
        .chars()
        .take(9)
        .collect::<String>();

    let essence = format!(
        "ayekvm seq={} class={} base243={} active={}/144",
        packet.frame_seq,
        packet.semantic_class.as_str(),
        cap.wave_b3,
        packet.active_glyph_cells
    );
    let wave = WaveFrame79::from_semantic_text(&essence);
    let wave_bytes = wave.to_bytes_79();
    let mut wave_hex = String::with_capacity(158);
    for &b in &wave_bytes {
        wave_hex.push_str(&format!("{b:02x}"));
    }

    json!({
        "address": MemnetAddress::default().to_canonical_string(),
        "capsule64": cap,
        "capsule64_hex": cap.to_hex(),
        "wave_frame79": wave,
        "wave_frame79_hex": wave_hex,
    })
}

/// Capture the live HDMI snapshot + board telemetry from `RustyNanoKVM` and analyze it with
/// the Balanced Ternary Video + PhoeniX `.8a∞` Audio Context Model + MemNet Capsule.
pub fn capture_live_ternary_context(
    analyzer: &mut TernaryVideoAnalyzer,
    addr_str: &str,
) -> Result<(TernaryMcpContextPacket, Value, Value), String> {
    let vm_info_bytes = http_get_bytes(addr_str, "/api/vm/info").unwrap_or_default();
    let vm_info: Value = serde_json::from_slice(&vm_info_bytes).unwrap_or_else(|_| json!({}));

    let jpeg_bytes = http_get_bytes(addr_str, "/api/stream/snapshot")?;
    let packet = analyzer.analyze_jpeg_frame(&jpeg_bytes, &[]);
    let memnet_ctx = build_memnet_context(&packet);
    Ok((packet, vm_info, memnet_ctx))
}

fn build_tools_list() -> Value {
    json!({
        "tools": [
            {
                "name": "rustykvm_get_ternary_context",
                "description": "Capture the live 1080p HDMI video frame, 80x25 LLM screen transcription buffer, PhoeniX .8a audio state, and 64-byte MemNet Capsule (0x4D4E) + 79-byte WaveFrame79 from RustyNanoKVM (rustykvm / ayekvm).",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "host": {
                            "type": "string",
                            "description": "Optional RustyNanoKVM host:port (defaults to rustykvm:80)"
                        }
                    }
                }
            },
            {
                "name": "rustykvm_send_hid_paste",
                "description": "Type text into the target machine via RustyNanoKVM USB HID (/api/hid/paste) and return the immediate post-input Balanced Ternary Video + Audio delta context.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "text": {
                            "type": "string",
                            "description": "Text to type over USB HID keyboard"
                        },
                        "host": {
                            "type": "string",
                            "description": "Optional RustyNanoKVM host:port (defaults to rustykvm:80)"
                        }
                    },
                    "required": ["text"]
                }
            },
            {
                "name": "rustykvm_send_hid_key",
                "description": "Send a USB HID keycode + modifier to the target machine via RustyNanoKVM (/api/hid/keyboard) and return the updated Balanced Ternary Video + Audio context.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "modifier": {
                            "type": "integer",
                            "description": "USB HID modifier bitmask (0 for none, 2 for Left Shift)"
                        },
                        "key": {
                            "type": "integer",
                            "description": "USB HID keycode (e.g. 40 for Enter, 41 for Escape, 76 for Delete/BIOS)"
                        },
                        "host": {
                            "type": "string",
                            "description": "Optional RustyNanoKVM host:port (defaults to rustykvm:80)"
                        }
                    },
                    "required": ["key"]
                }
            },
            {
                "name": "rustykvm_memnet_tree",
                "description": "Smart-Tree (st) × MemNet directory scan: returns compact ASCII tree, .m8 consciousness capsule count, IEEE CRC32 + FNV64-1a file manifests, 79-byte WaveFrame79, and 64-byte MemnetCapsule64.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Directory path to scan (defaults to '.')"
                        },
                        "depth": {
                            "type": "integer",
                            "description": "Max directory recursion depth (1..8, defaults to 3)"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Max files to index (1..512, defaults to 128)"
                        }
                    }
                }
            },
            {
                "name": "rustykvm_memnet_read",
                "description": "Read a bounded chunk of a file over MemNet with IEEE CRC32 + FNV64-1a integrity verification (like Smart-Tree read).",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "File path to read"
                        },
                        "offset": {
                            "type": "integer",
                            "description": "Byte offset (defaults to 0)"
                        },
                        "max_bytes": {
                            "type": "integer",
                            "description": "Maximum bytes to read (1..65536, defaults to 32768)"
                        }
                    },
                    "required": ["path"]
                }
            },
            {
                "name": "rustykvm_memnet_search",
                "description": "Fast Smart-Tree style keyword/symbol search across a directory tree over MemNet, scored with exact p/q rationals and CRC32 file digests.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Root directory path to search (defaults to '.')"
                        },
                        "query": {
                            "type": "string",
                            "description": "Keyword or symbol to search for"
                        },
                        "max_hits": {
                            "type": "integer",
                            "description": "Maximum search hits to return (1..100, defaults to 25)"
                        }
                    },
                    "required": ["query"]
                }
            },
            {
                "name": "rustykvm_memnet_broadcast",
                "description": "Capture the live Balanced Ternary Video + PhoeniX Audio state from RustyNanoKVM, seal it into a 64-byte MemnetCapsule64 (0x4D4E), and broadcast it over UDP to Smart-Tree (:8423, :8888) and AyeOS (:8432).",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "host": {
                            "type": "string",
                            "description": "Optional RustyNanoKVM host:port (defaults to rustykvm:80)"
                        }
                    }
                }
            }
        ]
    })
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let default_host =
        env::var("RUSTYKVM_HOST").unwrap_or_else(|_| DEFAULT_KVM_ADDR.to_string());
    let mut analyzer = TernaryVideoAnalyzer::new();

    if args.iter().any(|a| a == "--once") {
        let host = args
            .iter()
            .position(|a| a == "--host")
            .and_then(|i| args.get(i + 1))
            .map(|s| s.as_str())
            .unwrap_or(&default_host);

        match capture_live_ternary_context(&mut analyzer, host) {
            Ok((packet, vm_info, memnet_ctx)) => {
                let out = json!({
                    "ok": true,
                    "kvm_host": host,
                    "vm_info": vm_info.get("data").cloned().unwrap_or(vm_info),
                    "ternary_context": packet,
                    "memnet": memnet_ctx,
                });
                println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
            }
            Err(e) => {
                println!("{}", json!({ "ok": false, "kvm_host": host, "error": e }));
            }
        }
        return;
    }

    // Standard MCP JSON-RPC 2.0 stdio loop
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    for line in stdin.lock().lines().map_while(Result::ok) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let Ok(req) = serde_json::from_str::<Value>(trimmed) else {
            continue;
        };
        let id = req.get("id").cloned().unwrap_or(Value::Null);
        let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");

        let resp = match method {
            "initialize" => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": { "tools": {} },
                    "serverInfo": {
                        "name": "rustykvm-ternary-memnet-mcp",
                        "version": "0.2.0"
                    }
                }
            }),
            "notifications/initialized" => continue,
            "tools/list" => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": build_tools_list()
            }),
            "tools/call" => {
                let params = req.get("params").cloned().unwrap_or_else(|| json!({}));
                let tool_name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
                let tool_args = params.get("arguments").cloned().unwrap_or_else(|| json!({}));
                let host = tool_args
                    .get("host")
                    .and_then(|h| h.as_str())
                    .unwrap_or(&default_host);

                match tool_name {
                    "rustykvm_get_ternary_context" => match capture_live_ternary_context(&mut analyzer, host) {
                        Ok((packet, vm_info, memnet_ctx)) => {
                            let payload = json!({
                                "ok": true,
                                "kvm_host": host,
                                "vm_info": vm_info.get("data").cloned().unwrap_or(vm_info),
                                "ternary_context": packet,
                                "memnet": memnet_ctx,
                            });
                            json!({
                                "jsonrpc": "2.0",
                                "id": id,
                                "result": {
                                    "content": [{ "type": "text", "text": payload.to_string() }]
                                }
                            })
                        }
                        Err(e) => json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "result": {
                                "isError": true,
                                "content": [{ "type": "text", "text": e }]
                            }
                        }),
                    },
                    "rustykvm_send_hid_paste" => {
                        let text = tool_args
                            .get("text")
                            .and_then(|t| t.as_str())
                            .unwrap_or("");
                        let body = json!({ "text": text }).to_string();
                        let hid_res = http_post_json(host, "/api/hid/paste", &body);
                        let ctx = capture_live_ternary_context(&mut analyzer, host).ok();
                        let payload = json!({
                            "ok": hid_res.is_ok(),
                            "hid_response": hid_res.ok(),
                            "post_input_ternary_context": ctx.as_ref().map(|c| &c.0),
                            "memnet": ctx.as_ref().map(|c| &c.2),
                        });
                        json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "result": {
                                "content": [{ "type": "text", "text": payload.to_string() }]
                            }
                        })
                    }
                    "rustykvm_send_hid_key" => {
                        let modifier = tool_args
                            .get("modifier")
                            .and_then(|m| m.as_u64())
                            .unwrap_or(0) as u8;
                        let key = tool_args
                            .get("key")
                            .and_then(|k| k.as_u64())
                            .unwrap_or(0) as u8;
                        let body = json!({ "modifier": modifier, "keys": [key] }).to_string();
                        let hid_res = http_post_json(host, "/api/hid/keyboard", &body);
                        let ctx = capture_live_ternary_context(&mut analyzer, host).ok();
                        let payload = json!({
                            "ok": hid_res.is_ok(),
                            "hid_response": hid_res.ok(),
                            "post_input_ternary_context": ctx.as_ref().map(|c| &c.0),
                            "memnet": ctx.as_ref().map(|c| &c.2),
                        });
                        json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "result": {
                                "content": [{ "type": "text", "text": payload.to_string() }]
                            }
                        })
                    }
                    "rustykvm_memnet_tree" => {
                        let path = tool_args
                            .get("path")
                            .and_then(|p| p.as_str())
                            .unwrap_or(".");
                        let depth = tool_args
                            .get("depth")
                            .and_then(|d| d.as_u64())
                            .unwrap_or(3) as usize;
                        let limit = tool_args
                            .get("limit")
                            .and_then(|l| l.as_u64())
                            .unwrap_or(128) as usize;

                        match scan_smart_tree_directory(Path::new(path), depth, limit) {
                            Ok(manifest) => json!({
                                "jsonrpc": "2.0",
                                "id": id,
                                "result": {
                                    "content": [{ "type": "text", "text": json!({ "ok": true, "manifest": manifest }).to_string() }]
                                }
                            }),
                            Err(e) => json!({
                                "jsonrpc": "2.0",
                                "id": id,
                                "result": {
                                    "isError": true,
                                    "content": [{ "type": "text", "text": e }]
                                }
                            }),
                        }
                    }
                    "rustykvm_memnet_read" => {
                        let path = tool_args
                            .get("path")
                            .and_then(|p| p.as_str())
                            .unwrap_or("");
                        let offset = tool_args
                            .get("offset")
                            .and_then(|o| o.as_u64())
                            .unwrap_or(0) as usize;
                        let max_bytes = tool_args
                            .get("max_bytes")
                            .and_then(|m| m.as_u64())
                            .unwrap_or(32768) as usize;

                        match read_memnet_file_chunk(Path::new(path), offset, max_bytes) {
                            Ok(chunk) => json!({
                                "jsonrpc": "2.0",
                                "id": id,
                                "result": {
                                    "content": [{ "type": "text", "text": json!({ "ok": true, "chunk": chunk }).to_string() }]
                                }
                            }),
                            Err(e) => json!({
                                "jsonrpc": "2.0",
                                "id": id,
                                "result": {
                                    "isError": true,
                                    "content": [{ "type": "text", "text": e }]
                                }
                            }),
                        }
                    }
                    "rustykvm_memnet_search" => {
                        let path = tool_args
                            .get("path")
                            .and_then(|p| p.as_str())
                            .unwrap_or(".");
                        let query = tool_args
                            .get("query")
                            .and_then(|q| q.as_str())
                            .unwrap_or("");
                        let max_hits = tool_args
                            .get("max_hits")
                            .and_then(|m| m.as_u64())
                            .unwrap_or(25) as usize;

                        match search_smart_tree_files(Path::new(path), query, max_hits) {
                            Ok(hits) => json!({
                                "jsonrpc": "2.0",
                                "id": id,
                                "result": {
                                    "content": [{
                                        "type": "text",
                                        "text": json!({
                                            "ok": true,
                                            "query": query,
                                            "hit_count": hits.len(),
                                            "hits": hits
                                        }).to_string()
                                    }]
                                }
                            }),
                            Err(e) => json!({
                                "jsonrpc": "2.0",
                                "id": id,
                                "result": {
                                    "isError": true,
                                    "content": [{ "type": "text", "text": e }]
                                }
                            }),
                        }
                    }
                    "rustykvm_memnet_broadcast" => {
                        let mut cap = MemnetCapsule64::default();
                        if let Ok((pkt, _, _)) = capture_live_ternary_context(&mut analyzer, host) {
                            cap.seq = (pkt.frame_seq & 0xFFFF_FFFF) as u32;
                            cap.node_trit = if pkt.active_glyph_cells > 0 { 1 } else { 0 };
                            cap.wave_b3 = pkt.spatial_base243_hex.chars().take(9).collect();
                        }
                        match broadcast_memnet_capsule_udp(&cap) {
                            Ok(sent) => json!({
                                "jsonrpc": "2.0",
                                "id": id,
                                "result": {
                                    "content": [{
                                        "type": "text",
                                        "text": json!({
                                            "ok": true,
                                            "udp_packets_sent": sent,
                                            "ports": [8423, 8432, 8888],
                                            "capsule64_hex": cap.to_hex(),
                                        }).to_string()
                                    }]
                                }
                            }),
                            Err(e) => json!({
                                "jsonrpc": "2.0",
                                "id": id,
                                "result": {
                                    "isError": true,
                                    "content": [{ "type": "text", "text": e }]
                                }
                            }),
                        }
                    }
                    _ => json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": { "code": -32601, "message": "Unknown tool" }
                    }),
                }
            }
            _ => json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32601, "message": "Method not found" }
            }),
        };

        let _ = writeln!(stdout, "{resp}");
        let _ = stdout.flush();
    }
}
