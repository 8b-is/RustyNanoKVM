#![forbid(unsafe_code)]
//! Sovereign `MEMNET` Protocol, Smart-Tree Shared Folder & File Chunk Access,
//! 64-Byte `MemnetCapsule64` (`0x4D4E`), and 79-Byte `WaveFrame79` (`MEM|8` v2) for `RustyNanoKVM`.
//!
//! Bridges `RustyNanoKVM` (`ayekvm`) directly into the `Smart-Tree` (`st`) and `ayeOS` mesh:
//! - **Contextual Addressing (`MemnetAddress`)**: `geo:sanctuary + role:kvm.ternary.sg2002 + tags:hdmi,hid,memnet,smart-tree`
//! - **Byte-Exact 64-Byte `MemnetCapsule64` (`0x4D4E` `"NM"`)**: Sealed with 64-bit FNV-1a at `buf[56..64]`,
//!   carrying node trit `{-1, 0, +1}`, PhoeniX audio `F0` Hz & salience, and Base-243 ternary wave digest.
//! - **Byte-Exact 79-Byte `WaveFrame79` (`MEM|8` v2)**: Integer `p/q` rational cognitive wave frame.
//! - **Smart-Tree File Tree, `.m8` Consciousness, Read & Search (`SharedFolderManifest`)**:
//!   Exposes directory trees, `.m8` consciousness files, IEEE CRC32 + FNV64-1a file digests,
//!   32KB `MediaChunkPacket` streaming, and fast keyword/symbol search to MCP agents and MemNet peers.
//!
//! **Strict 8b-is No-Float Law: 100% integer, balanced ternary `{-1, 0, +1}`, and reduced `p/q` rationals.**

use std::fs;
use std::net::UdpSocket;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// Default MemNet Semantic Filesystem & Smart-Tree Share port (`8423`).
pub const MEMNET_FS_PORT: u16 = 8423;
/// Default AyeOS × OxiCast Sanctuary port (`8432`).
pub const MEMNET_AYEOS_PORT: u16 = 8432;
/// Default ESP32 Constellation & Smart-Tree UDP mesh port (`8888`).
pub const MEMNET_ESP32_PORT: u16 = 8888;

/// Magic header (`0x4D4E`, little-endian `'N','M'`) for 64-byte `MemnetCapsule64` packets.
pub const MEMNET_CAPSULE64_MAGIC: u16 = 0x4D4E;

/// Default chunk size (`32 KB`) for MemNet file and media streaming.
pub const MEMNET_CHUNK_BYTES: usize = 32 * 1024;

/// FNV-1a 64-bit offset basis (`0xCBF29CE484222325`).
pub const FNV64_BASIS: u64 = 0xCBF2_9CE4_8422_2325;
/// FNV-1a 64-bit prime (`0x00000100000001B3`).
pub const FNV64_PRIME: u64 = 0x0000_0100_0000_01B3;

/// FNV-1a 32-bit offset basis (`0x811C9DC5`).
pub const FNV32_BASIS: u32 = 0x811C_9DC5;
/// FNV-1a 32-bit prime (`0x01000193`).
pub const FNV32_PRIME: u32 = 0x0100_0193;

/// Pure-integer 64-bit FNV-1a hash matching `smart-tree/src/memnet.rs`.
#[inline]
pub const fn fnv64_1a(data: &[u8]) -> u64 {
    let mut h = FNV64_BASIS;
    let mut i = 0;
    while i < data.len() {
        h ^= data[i] as u64;
        h = h.wrapping_mul(FNV64_PRIME);
        i += 1;
    }
    h
}

/// Pure-integer 32-bit FNV-1a hash matching `smart-tree/src/memnet.rs`.
#[inline]
pub const fn fnv32_1a(data: &[u8]) -> u32 {
    let mut h = FNV32_BASIS;
    let mut i = 0;
    while i < data.len() {
        h ^= data[i] as u32;
        h = h.wrapping_mul(FNV32_PRIME);
        i += 1;
    }
    h
}

/// Pure-integer IEEE 802.3 CRC-32 (`0xEDB88320`) matching `smart-tree` `felix_crc32_ieee`.
#[inline]
pub const fn crc32_ieee(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    let mut i = 0;
    while i < data.len() {
        crc ^= data[i] as u32;
        let mut bit = 0;
        while bit < 8 {
            if (crc & 1) != 0 {
                crc = (crc >> 1) ^ 0xEDB8_8320;
            } else {
                crc >>= 1;
            }
            bit += 1;
        }
        i += 1;
    }
    !crc
}

#[inline]
const fn gcd_u32(mut a: u32, mut b: u32) -> u32 {
    if a == 0 {
        return if b == 0 { 1 } else { b };
    }
    if b == 0 {
        return a;
    }
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    a
}

/// Contextual MemNet address (`geo:... + role:... + tags:a,b,c + vector:...`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemnetAddress {
    pub geo: Option<String>,
    pub role: String,
    pub tags: Vec<String>,
    pub vector_hash: Option<String>,
}

impl Default for MemnetAddress {
    fn default() -> Self {
        Self::parse(
            "geo:sanctuary + role:kvm.ternary.sg2002 + tags:ayekvm,hdmi,hid,ternary,phoenix,smart-tree,memnet",
        )
    }
}

impl MemnetAddress {
    pub fn parse(expr: &str) -> Self {
        let mut geo = None;
        let mut role = String::new();
        let mut tags = Vec::new();
        let mut vector_hash = None;

        for part in expr.split('+').map(str::trim).filter(|s| !s.is_empty()) {
            if let Some(g) = part.strip_prefix("geo:") {
                geo = Some(g.trim().to_string());
            } else if let Some(r) = part.strip_prefix("role:") {
                role = r.trim().to_string();
            } else if let Some(ts) = part.strip_prefix("tags:").or_else(|| part.strip_prefix("tag:")) {
                for t in ts.split(',').map(str::trim).filter(|s| !s.is_empty()) {
                    tags.push(t.to_string());
                }
            } else if let Some(v) = part.strip_prefix("vector:") {
                vector_hash = Some(v.trim().to_string());
            }
        }

        Self {
            geo,
            role,
            tags,
            vector_hash,
        }
    }

    pub fn to_canonical_string(&self) -> String {
        let mut parts = Vec::new();
        if let Some(ref g) = self.geo {
            parts.push(format!("geo:{g}"));
        }
        if !self.role.is_empty() {
            parts.push(format!("role:{}", self.role));
        }
        if !self.tags.is_empty() {
            parts.push(format!("tags:{}", self.tags.join(",")));
        }
        if let Some(ref v) = self.vector_hash {
            parts.push(format!("vector:{v}"));
        }
        parts.join(" + ")
    }
}

/// Exact 8-byte integer rational (`i32` numerator, positive `u32` denominator) for `WaveFrame79`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WaveRational {
    pub num: i32,
    pub den: u32,
}

impl WaveRational {
    pub const fn new(num: i32, den: u32) -> Option<Self> {
        if den == 0 {
            return None;
        }
        let g = gcd_u32(num.unsigned_abs(), den);
        Some(Self {
            num: num / (g as i32),
            den: den / g,
        })
    }

    pub const fn to_bytes(self) -> [u8; 8] {
        let mut out = [0u8; 8];
        let n = self.num.to_le_bytes();
        let d = self.den.to_le_bytes();
        out[0] = n[0];
        out[1] = n[1];
        out[2] = n[2];
        out[3] = n[3];
        out[4] = d[0];
        out[5] = d[1];
        out[6] = d[2];
        out[7] = d[3];
        out
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 8 {
            return None;
        }
        let num = i32::from_le_bytes(bytes[0..4].try_into().ok()?);
        let den = u32::from_le_bytes(bytes[4..8].try_into().ok()?);
        Self::new(num, den)
    }

    pub fn format_pq(&self) -> String {
        format!("{}/{}", self.num, self.den)
    }
}

/// Byte-exact 79-byte `MEM|8` Version 2 Integer Wire Frame matching `smart-tree/src/memnet.rs`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WaveFrame79 {
    pub version: u8,
    pub provenance: u8,
    pub base_amplitude: WaveRational,
    pub frequency: WaveRational,
    pub phase: WaveRational,
    pub emotional_valence: WaveRational,
    pub arousal: WaveRational,
    pub half_life_ticks: u64,
    pub age_ticks: u64,
    pub idle_ticks: u64,
    pub access_count: u32,
    pub id: Option<u64>,
}

impl WaveFrame79 {
    pub fn to_bytes_79(&self) -> [u8; 79] {
        let mut buf = [0u8; 79];
        buf[0] = 2;
        buf[1] = if self.provenance == 0x02 { 0x02 } else { 0x01 };
        buf[2..10].copy_from_slice(&self.base_amplitude.to_bytes());
        buf[10..18].copy_from_slice(&self.frequency.to_bytes());
        buf[18..26].copy_from_slice(&self.phase.to_bytes());
        buf[26..34].copy_from_slice(&self.emotional_valence.to_bytes());
        buf[34..42].copy_from_slice(&self.arousal.to_bytes());
        buf[42..50].copy_from_slice(&self.half_life_ticks.to_le_bytes());
        buf[50..58].copy_from_slice(&self.age_ticks.to_le_bytes());
        buf[58..66].copy_from_slice(&self.idle_ticks.min(self.age_ticks).to_le_bytes());
        buf[66..70].copy_from_slice(&self.access_count.to_le_bytes());
        if let Some(id) = self.id {
            buf[70] = 1;
            buf[71..79].copy_from_slice(&id.to_le_bytes());
        } else {
            buf[70] = 0;
        }
        buf
    }

    pub fn from_bytes_79(buf: &[u8]) -> Option<Self> {
        if buf.len() < 79 {
            return None;
        }
        let version = buf[0];
        if version != 1 && version != 2 {
            return None;
        }
        let provenance = buf[1];
        if provenance != 0x01 && provenance != 0x02 {
            return None;
        }
        let base_amplitude = WaveRational::from_bytes(&buf[2..10])?;
        let frequency = WaveRational::from_bytes(&buf[10..18])?;
        let phase = WaveRational::from_bytes(&buf[18..26])?;
        let emotional_valence = WaveRational::from_bytes(&buf[26..34])?;
        let arousal = WaveRational::from_bytes(&buf[34..42])?;
        let half_life_ticks = u64::from_le_bytes(buf[42..50].try_into().ok()?);
        let age_ticks = u64::from_le_bytes(buf[50..58].try_into().ok()?);
        let idle_ticks = u64::from_le_bytes(buf[58..66].try_into().ok()?);
        if version == 2 && idle_ticks > age_ticks {
            return None;
        }
        let access_count = u32::from_le_bytes(buf[66..70].try_into().ok()?);
        let id = if buf[70] != 0 {
            Some(u64::from_le_bytes(buf[71..79].try_into().ok()?))
        } else {
            None
        };
        Some(Self {
            version: 2,
            provenance,
            base_amplitude,
            frequency,
            phase,
            emotional_valence,
            arousal,
            half_life_ticks,
            age_ticks,
            idle_ticks: idle_ticks.min(age_ticks),
            access_count,
            id,
        })
    }

    pub fn from_semantic_text(text: &str) -> Self {
        let fold = fnv64_1a(text.as_bytes());
        let freq_hz = 40 + ((fold % 400) as i32);
        let phase_deg = ((fold >> 16) % 360) as i32;
        let valence = (((fold >> 32) % 180) as i32) - 90;
        let arousal = 30 + (((fold >> 48) % 70) as i32);
        Self {
            version: 2,
            provenance: 0x01,
            base_amplitude: WaveRational::new(95, 100).unwrap(),
            frequency: WaveRational::new(freq_hz, 1).unwrap(),
            phase: WaveRational::new(phase_deg, 360).unwrap(),
            emotional_valence: WaveRational::new(valence, 100).unwrap(),
            arousal: WaveRational::new(arousal, 100).unwrap(),
            half_life_ticks: 86_400,
            age_ticks: 1,
            idle_ticks: 0,
            access_count: 1,
            id: Some(fold),
        }
    }
}

/// Byte-exact 64-byte `MemnetCapsule64` (`0x4D4E` `"NM"`) matching `smart-tree/src/memnet.rs`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemnetCapsule64 {
    pub version: u8,
    pub node_id: u8,
    pub node_trit: i8,
    pub transport_mask: u8,
    pub seed_class: u8,
    pub dominant_mod: u8,
    pub seq: u32,
    pub audio_f0_hz: u16,
    pub audio_salience_pct: u8,
    pub rf_rssi_dbm: i8,
    pub rf_ap_count: u16,
    pub imu_motion_mg: i16,
    pub peer_rssi_dbm: [i8; 4],
    pub top_bssid_fnv32: u32,
    pub top_bssid_rssi_dbm: i8,
    pub portal_ok: u8,
    pub wave_b3: String,
    pub ssid_hint: String,
}

impl Default for MemnetCapsule64 {
    fn default() -> Self {
        Self {
            version: 1,
            node_id: 103, // `ayekvm` / `rustykvm` MemNet node ID
            node_trit: 1,
            transport_mask: 0x02, // Ethernet/WiFi
            seed_class: 1,
            dominant_mod: 2, // Ternary Video + PhoeniX Audio
            seq: 1,
            audio_f0_hz: 432,
            audio_salience_pct: 94,
            rf_rssi_dbm: -42,
            rf_ap_count: 4,
            imu_motion_mg: 0,
            peer_rssi_dbm: [-40, -45, -50, -48],
            top_bssid_fnv32: fnv32_1a(b"ayekvm.memnet"),
            top_bssid_rssi_dbm: -40,
            portal_ok: 1,
            wave_b3: "360000eaf".to_string(),
            ssid_hint: "ayekvm".to_string(),
        }
    }
}

impl MemnetCapsule64 {
    pub fn to_bytes_64(&self) -> [u8; 64] {
        let mut buf = [0u8; 64];
        buf[0..2].copy_from_slice(&MEMNET_CAPSULE64_MAGIC.to_le_bytes());
        buf[2] = self.version;
        buf[3] = self.node_id;
        buf[4] = self.node_trit as u8;
        buf[5] = self.transport_mask;
        buf[6] = self.seed_class;
        buf[7] = self.dominant_mod;
        buf[8..12].copy_from_slice(&self.seq.to_le_bytes());
        buf[12..14].copy_from_slice(&self.audio_f0_hz.to_le_bytes());
        buf[14] = self.audio_salience_pct;
        buf[15] = self.rf_rssi_dbm as u8;
        buf[16..18].copy_from_slice(&self.rf_ap_count.to_le_bytes());
        buf[18..20].copy_from_slice(&self.imu_motion_mg.to_le_bytes());
        for i in 0..4 {
            buf[20 + i] = self.peer_rssi_dbm[i] as u8;
        }
        buf[24..28].copy_from_slice(&self.top_bssid_fnv32.to_le_bytes());
        buf[28] = self.top_bssid_rssi_dbm as u8;
        buf[29] = self.portal_ok;

        let b3 = self.wave_b3.as_bytes();
        let b3_len = b3.len().min(9);
        buf[30..30 + b3_len].copy_from_slice(&b3[..b3_len]);

        let ssid = self.ssid_hint.as_bytes();
        let ssid_len = ssid.len().min(15);
        buf[40..40 + ssid_len].copy_from_slice(&ssid[..ssid_len]);

        let seal = fnv64_1a(&buf[0..56]);
        buf[56..64].copy_from_slice(&seal.to_le_bytes());
        buf
    }

    pub fn from_bytes_64(buf: &[u8; 64]) -> Option<Self> {
        let magic = u16::from_le_bytes([buf[0], buf[1]]);
        if magic != MEMNET_CAPSULE64_MAGIC {
            return None;
        }
        let expected_seal = fnv64_1a(&buf[0..56]);
        let actual_seal = u64::from_le_bytes(buf[56..64].try_into().ok()?);
        if expected_seal != actual_seal {
            return None;
        }
        let b3_end = buf[30..40].iter().position(|&b| b == 0).unwrap_or(9).min(9);
        let wave_b3 = String::from_utf8_lossy(&buf[30..30 + b3_end]).to_string();
        let ssid_end = buf[40..56]
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(15)
            .min(15);
        let ssid_hint = String::from_utf8_lossy(&buf[40..40 + ssid_end]).to_string();

        Some(Self {
            version: buf[2],
            node_id: buf[3],
            node_trit: buf[4] as i8,
            transport_mask: buf[5],
            seed_class: buf[6],
            dominant_mod: buf[7],
            seq: u32::from_le_bytes([buf[8], buf[9], buf[10], buf[11]]),
            audio_f0_hz: u16::from_le_bytes([buf[12], buf[13]]),
            audio_salience_pct: buf[14],
            rf_rssi_dbm: buf[15] as i8,
            rf_ap_count: u16::from_le_bytes([buf[16], buf[17]]),
            imu_motion_mg: i16::from_le_bytes([buf[18], buf[19]]),
            peer_rssi_dbm: [buf[20] as i8, buf[21] as i8, buf[22] as i8, buf[23] as i8],
            top_bssid_fnv32: u32::from_le_bytes([buf[24], buf[25], buf[26], buf[27]]),
            top_bssid_rssi_dbm: buf[28] as i8,
            portal_ok: buf[29],
            wave_b3,
            ssid_hint,
        })
    }

    pub fn to_hex(&self) -> String {
        bytes_to_hex(&self.to_bytes_64())
    }
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// A single file entry inside a Smart-Tree `SharedFolderManifest`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SharedFileEntry {
    pub rel_path: String,
    pub size_bytes: u64,
    pub crc32: u32,
    pub fnv64: u64,
    pub chunk_count: u32,
    pub has_m8_sibling: bool,
}

/// A Smart-Tree compatible folder manifest shared over MemNet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SharedFolderManifest {
    pub share_id: String,
    pub share_name: String,
    pub host_path: String,
    pub owner_node: u8,
    pub owner_name: String,
    pub address: MemnetAddress,
    pub total_files: usize,
    pub total_dirs: usize,
    pub total_bytes: u64,
    pub m8_capsules_found: usize,
    pub semantic_essence: String,
    pub wave_frame_hex: String,
    pub capsule64_hex: String,
    pub compact_tree: Vec<String>,
    pub files: Vec<SharedFileEntry>,
    pub shared_at_s: u64,
}

/// Scan a directory in Smart-Tree (`st`) format and construct a `SharedFolderManifest`
/// with `.m8` consciousness detection, compact ASCII tree, and CRC32/FNV64 file manifests.
pub fn scan_smart_tree_directory(
    root_dir: &Path,
    max_depth: usize,
    max_files: usize,
) -> Result<SharedFolderManifest, String> {
    let canon = fs::canonicalize(root_dir)
        .map_err(|e| format!("cannot canonicalize {}: {e}", root_dir.display()))?;
    if !canon.is_dir() {
        return Err(format!("{} is not a directory", canon.display()));
    }

    let mut files = Vec::new();
    let mut compact_tree = Vec::new();
    let mut total_dirs = 0usize;
    let mut total_bytes = 0u64;
    let mut m8_capsules_found = 0usize;

    walk_smart_tree_recursive(
        &canon,
        &canon,
        0,
        max_depth.clamp(1, 8),
        max_files.clamp(1, 512),
        &mut files,
        &mut compact_tree,
        &mut total_dirs,
        &mut total_bytes,
        &mut m8_capsules_found,
    );

    files.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));

    let folder_name = canon
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "root".to_string());

    let semantic_essence = format!(
        "Smart-Tree MemNet Share '{folder_name}': {} files, {} dirs, {} .m8 capsules",
        files.len(),
        total_dirs,
        m8_capsules_found
    );
    let wave = WaveFrame79::from_semantic_text(&semantic_essence);
    let mut cap = MemnetCapsule64::default();
    cap.wave_b3 = format!("{:09x}", wave.id.unwrap_or(0) & 0xFFFF_FFFF);

    let share_id = format!(
        "share-{:08x}",
        fnv32_1a(format!("103:{}", canon.display()).as_bytes())
    );
    let shared_at_s = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    Ok(SharedFolderManifest {
        share_id,
        share_name: folder_name,
        host_path: canon.to_string_lossy().into_owned(),
        owner_node: 103,
        owner_name: "ayekvm".to_string(),
        address: MemnetAddress::default(),
        total_files: files.len(),
        total_dirs,
        total_bytes,
        m8_capsules_found,
        semantic_essence,
        wave_frame_hex: bytes_to_hex(&wave.to_bytes_79()),
        capsule64_hex: cap.to_hex(),
        compact_tree,
        files,
        shared_at_s,
    })
}

#[allow(clippy::too_many_arguments)]
fn walk_smart_tree_recursive(
    root: &Path,
    current: &Path,
    depth: usize,
    max_depth: usize,
    max_files: usize,
    files: &mut Vec<SharedFileEntry>,
    compact_tree: &mut Vec<String>,
    total_dirs: &mut usize,
    total_bytes: &mut u64,
    m8_capsules_found: &mut usize,
) {
    if depth > max_depth || files.len() >= max_files {
        return;
    }
    let Ok(rd) = fs::read_dir(current) else {
        return;
    };
    let mut entries: Vec<_> = rd.flatten().collect();
    entries.sort_by_key(|e| e.file_name());

    let has_local_m8 = current.join(".m8").exists();

    for entry in entries {
        if files.len() >= max_files {
            break;
        }
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str == ".git" || name_str == "target" || name_str == "node_modules" {
            continue;
        }
        let path = entry.path();
        let Ok(meta) = entry.metadata() else {
            continue;
        };
        let indent = "  ".repeat(depth);

        if meta.is_dir() {
            *total_dirs += 1;
            compact_tree.push(format!("{indent}📁 {name_str}/"));
            walk_smart_tree_recursive(
                root,
                &path,
                depth + 1,
                max_depth,
                max_files,
                files,
                compact_tree,
                total_dirs,
                total_bytes,
                m8_capsules_found,
            );
        } else if meta.is_file() {
            if name_str.ends_with(".m8") {
                *m8_capsules_found += 1;
            }
            let size_bytes = meta.len();
            *total_bytes = total_bytes.saturating_add(size_bytes);
            let rel = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");

            let sample = fs::read(&path)
                .map(|b| {
                    if b.len() > 65536 {
                        b[..65536].to_vec()
                    } else {
                        b
                    }
                })
                .unwrap_or_default();
            let crc32 = crc32_ieee(&sample);
            let fnv64 = fnv64_1a(&sample);
            let chunk_count = (size_bytes as usize).div_ceil(MEMNET_CHUNK_BYTES).max(1) as u32;

            compact_tree.push(format!("{indent}📄 {name_str} ({size_bytes}B, crc32={crc32:08x})"));
            files.push(SharedFileEntry {
                rel_path: rel,
                size_bytes,
                crc32,
                fnv64,
                chunk_count,
                has_m8_sibling: has_local_m8,
            });
        }
    }
}

/// Chunked file read response sealed with IEEE CRC32 and FNV64-1a.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemnetFileChunk {
    pub path: String,
    pub total_size_bytes: u64,
    pub offset: usize,
    pub chunk_len: usize,
    pub crc32: u32,
    pub fnv64: u64,
    pub is_utf8: bool,
    pub content_text: String,
    pub content_hex: String,
}

/// Read a bounded chunk of a file over MemNet (`max_bytes` capped at `65536`) with CRC32 + FNV64 verification.
pub fn read_memnet_file_chunk(
    path: &Path,
    offset: usize,
    max_bytes: usize,
) -> Result<MemnetFileChunk, String> {
    let canon = fs::canonicalize(path)
        .map_err(|e| format!("cannot access {}: {e}", path.display()))?;
    if !canon.is_file() {
        return Err(format!("{} is not a regular file", canon.display()));
    }
    let raw = fs::read(&canon).map_err(|e| format!("read {} failed: {e}", canon.display()))?;
    let total_size_bytes = raw.len() as u64;
    let start = offset.min(raw.len());
    let limit = max_bytes.clamp(1, 65536);
    let end = (start + limit).min(raw.len());
    let slice = &raw[start..end];

    let crc32 = crc32_ieee(slice);
    let fnv64 = fnv64_1a(slice);
    let (is_utf8, content_text) = match std::str::from_utf8(slice) {
        Ok(s) => (true, s.to_string()),
        Err(_) => (false, String::from_utf8_lossy(slice).into_owned()),
    };

    Ok(MemnetFileChunk {
        path: canon.to_string_lossy().into_owned(),
        total_size_bytes,
        offset: start,
        chunk_len: slice.len(),
        crc32,
        fnv64,
        is_utf8,
        content_text,
        content_hex: if is_utf8 {
            String::new()
        } else {
            bytes_to_hex(slice)
        },
    })
}

/// A single Smart-Tree style search match over MemNet, scored with an exact `p/q` rational.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemnetSearchHit {
    pub file_path: String,
    pub line_number: usize,
    pub line_preview: String,
    pub relevance_pq: String,
    pub file_crc32: u32,
}

/// Fast Smart-Tree style keyword/symbol search across a directory tree.
pub fn search_smart_tree_files(
    root_dir: &Path,
    query: &str,
    max_hits: usize,
) -> Result<Vec<MemnetSearchHit>, String> {
    let needle = query.trim().to_ascii_lowercase();
    if needle.is_empty() {
        return Ok(Vec::new());
    }
    let manifest = scan_smart_tree_directory(root_dir, 5, 256)?;
    let root_path = PathBuf::from(&manifest.host_path);
    let mut hits = Vec::new();
    let cap_hits = max_hits.clamp(1, 100);

    for f in &manifest.files {
        if hits.len() >= cap_hits {
            break;
        }
        let full = root_path.join(&f.rel_path);
        // Check filename match first
        if f.rel_path.to_ascii_lowercase().contains(&needle) {
            hits.push(MemnetSearchHit {
                file_path: f.rel_path.clone(),
                line_number: 0,
                line_preview: format!("[filename match] {}", f.rel_path),
                relevance_pq: "15/16".to_string(),
                file_crc32: f.crc32,
            });
        }
        if f.size_bytes > 512 * 1024 {
            continue;
        }
        let Ok(text) = fs::read_to_string(&full) else {
            continue;
        };
        for (idx, line) in text.lines().enumerate() {
            if hits.len() >= cap_hits {
                break;
            }
            if line.to_ascii_lowercase().contains(&needle) {
                let preview: String = line.trim().chars().take(140).collect();
                let score_num = if f.has_m8_sibling { 960 } else { 832 };
                let g = gcd_u32(score_num, 1024);
                hits.push(MemnetSearchHit {
                    file_path: f.rel_path.clone(),
                    line_number: idx + 1,
                    line_preview: preview,
                    relevance_pq: format!("{}/{}", score_num / g, 1024 / g),
                    file_crc32: f.crc32,
                });
            }
        }
    }

    Ok(hits)
}

/// Broadcast a 64-byte `MemnetCapsule64` (`0x4D4E`) over UDP to Smart-Tree (`:8423`, `:8888`)
/// and AyeOS Sanctuary (`:8432`) peers.
pub fn broadcast_memnet_capsule_udp(capsule: &MemnetCapsule64) -> Result<usize, String> {
    let sock = UdpSocket::bind("0.0.0.0:0").map_err(|e| format!("udp bind failed: {e}"))?;
    let _ = sock.set_broadcast(true);
    let pkt = capsule.to_bytes_64();
    let mut sent = 0usize;
    for port in [MEMNET_FS_PORT, MEMNET_AYEOS_PORT, MEMNET_ESP32_PORT] {
        if sock.send_to(&pkt, ("127.0.0.1", port)).is_ok() {
            sent += 1;
        }
        let _ = sock.send_to(&pkt, ("255.255.255.255", port));
    }
    Ok(sent)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memnet_capsule64_and_waveframe79_roundtrip_exact() {
        let cap = MemnetCapsule64::default();
        let bytes = cap.to_bytes_64();
        assert_eq!(&bytes[0..2], b"NM"); // 0x4D4E little-endian
        let decoded = MemnetCapsule64::from_bytes_64(&bytes).expect("valid 64B capsule");
        assert_eq!(decoded, cap);

        let wave = WaveFrame79::from_semantic_text("ayekvm ternary video + smart-tree memnet");
        let wbytes = wave.to_bytes_79();
        let wdecoded = WaveFrame79::from_bytes_79(&wbytes).expect("valid 79B WaveFrame");
        assert_eq!(wdecoded, wave);
    }

    #[test]
    fn smart_tree_directory_scan_and_search_work_cleanly() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let manifest = scan_smart_tree_directory(dir, 3, 64).expect("scan crate dir");
        assert!(manifest.total_files > 0);
        assert!(!manifest.compact_tree.is_empty());

        let hits = search_smart_tree_files(dir, "MemnetCapsule64", 10).expect("search crate dir");
        assert!(!hits.is_empty());
    }
}
