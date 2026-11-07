#![forbid(unsafe_code)]
//! MemNet & Smart-Tree (`st`) File/Context API handlers for `RustyNanoKVM` (`ayekvm`).
//!
//! Exposes:
//! - `GET /api/memnet/capsule`: Live 64-byte `MemnetCapsule64` (`0x4D4E`) + 79-byte `WaveFrame79` (`MEM|8` v2)
//!   fused with the current Balanced Ternary Video (`Base-243`) and PhoeniX `.8a∞` Audio state.
//! - `GET /api/memnet/tree`: Smart-Tree directory scan (`SharedFolderManifest`) with `.m8` consciousness detection,
//!   compact tree rendering, and IEEE CRC32 + FNV64-1a file digests.
//! - `GET /api/memnet/read`: Bounded 32KB/64KB chunked file read with CRC32 + FNV64 verification.
//! - `GET /api/memnet/search`: Fast Smart-Tree keyword/symbol search with reduced `p/q` rational relevance.
//! - `POST /api/memnet/broadcast`: Broadcasts the 64-byte `MemnetCapsule64` over UDP to `:8423`, `:8432`, `:8888`.

use std::path::Path;
use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use nanokvm_core::memnet::{
    MemnetAddress, MemnetCapsule64, WaveFrame79, broadcast_memnet_capsule_udp,
    read_memnet_file_chunk, scan_smart_tree_directory, search_smart_tree_files,
};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct TreeQuery {
    pub path: Option<String>,
    pub depth: Option<usize>,
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct ReadQuery {
    pub path: String,
    pub offset: Option<usize>,
    pub max_bytes: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub path: Option<String>,
    pub q: String,
    pub max_hits: Option<usize>,
}

fn build_live_capsule_and_wave(state: &AppState) -> (MemnetCapsule64, WaveFrame79) {
    let jpeg_bytes = state
        .video
        .read()
        .read_mjpeg(1920, 1080, 80)
        .map(|f| f.data)
        .unwrap_or_default();
    let pkt = state
        .ternary_analyzer
        .write()
        .analyze_jpeg_frame(&jpeg_bytes, &[]);

    let mut cap = MemnetCapsule64::default();
    cap.seq = (pkt.frame_seq & 0xFFFF_FFFF) as u32;
    cap.node_trit = if pkt.active_glyph_cells > 0 { 1 } else { 0 };
    cap.wave_b3 = pkt
        .spatial_base243_hex
        .chars()
        .take(9)
        .collect::<String>();

    let essence = format!(
        "ayekvm seq={} class={} base243={} active={}/144",
        pkt.frame_seq,
        pkt.semantic_class.as_str(),
        cap.wave_b3,
        pkt.active_glyph_cells
    );
    let wave = WaveFrame79::from_semantic_text(&essence);
    (cap, wave)
}

/// `GET /api/memnet/capsule` — Return the live 64B `MemnetCapsule64` and 79B `WaveFrame79`.
pub async fn get_capsule(
    State(state): State<Arc<AppState>>,
) -> (StatusCode, Json<Value>) {
    let (cap, wave) = build_live_capsule_and_wave(&state);
    let wave_bytes = wave.to_bytes_79();
    let mut wave_hex = String::with_capacity(158);
    for &b in &wave_bytes {
        wave_hex.push_str(&format!("{b:02x}"));
    }

    (
        StatusCode::OK,
        Json(json!({
            "code": 0,
            "data": {
                "address": MemnetAddress::default().to_canonical_string(),
                "capsule64": cap,
                "capsule64_hex": cap.to_hex(),
                "wave_frame79": wave,
                "wave_frame79_hex": wave_hex,
            }
        })),
    )
}

/// `GET /api/memnet/tree` — Smart-Tree style directory scan (`SharedFolderManifest`).
pub async fn get_tree(
    Query(query): Query<TreeQuery>,
) -> (StatusCode, Json<Value>) {
    let root = query.path.as_deref().unwrap_or(".");
    let depth = query.depth.unwrap_or(3);
    let limit = query.limit.unwrap_or(128);

    match scan_smart_tree_directory(Path::new(root), depth, limit) {
        Ok(manifest) => (
            StatusCode::OK,
            Json(json!({
                "code": 0,
                "data": manifest,
            })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "code": -1,
                "msg": e,
            })),
        ),
    }
}

/// `GET /api/memnet/read` — Read a CRC32/FNV64-verified chunk of a file over MemNet.
pub async fn read_file(
    Query(query): Query<ReadQuery>,
) -> (StatusCode, Json<Value>) {
    let offset = query.offset.unwrap_or(0);
    let max_bytes = query.max_bytes.unwrap_or(32 * 1024);

    match read_memnet_file_chunk(Path::new(&query.path), offset, max_bytes) {
        Ok(chunk) => (
            StatusCode::OK,
            Json(json!({
                "code": 0,
                "data": chunk,
            })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "code": -1,
                "msg": e,
            })),
        ),
    }
}

/// `GET /api/memnet/search` — Fast Smart-Tree keyword/symbol search across files.
pub async fn search_files(
    Query(query): Query<SearchQuery>,
) -> (StatusCode, Json<Value>) {
    let root = query.path.as_deref().unwrap_or(".");
    let max_hits = query.max_hits.unwrap_or(25);

    match search_smart_tree_files(Path::new(root), &query.q, max_hits) {
        Ok(hits) => (
            StatusCode::OK,
            Json(json!({
                "code": 0,
                "data": {
                    "query": query.q,
                    "root": root,
                    "hit_count": hits.len(),
                    "hits": hits,
                }
            })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "code": -1,
                "msg": e,
            })),
        ),
    }
}

/// `POST /api/memnet/broadcast` — Broadcast the 64B `MemnetCapsule64` over UDP (`:8423`, `:8432`, `:8888`).
pub async fn broadcast_capsule(
    State(state): State<Arc<AppState>>,
) -> (StatusCode, Json<Value>) {
    let (cap, _wave) = build_live_capsule_and_wave(&state);
    match broadcast_memnet_capsule_udp(&cap) {
        Ok(sent) => (
            StatusCode::OK,
            Json(json!({
                "code": 0,
                "data": {
                    "udp_packets_sent": sent,
                    "ports": [8423, 8432, 8888],
                    "capsule64_hex": cap.to_hex(),
                }
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "code": -1,
                "msg": e,
            })),
        ),
    }
}
