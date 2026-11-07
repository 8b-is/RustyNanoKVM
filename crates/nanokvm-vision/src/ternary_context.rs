#![forbid(unsafe_code)]
//! Balanced Ternary (`{-1, 0, +1}`) Video & PhoeniX `.8a∞` Audio Context Model for `RustyNanoKVM`.
//!
//! Provides AI agents and MCP servers with compact, high-salience structural context of both
//! the captured 1080p HDMI video stream and audio stream without shipping multi-megabyte raw
//! pixel/PCM buffers:
//! - **Spatial & Temporal Ternary Bitplanes (`16 × 9 = 144` trits)**:
//!   - `-1`: Dark void / decaying energy
//!   - `0`: Static surface / temporal deadband (`0x20 LINEUP`)
//!   - `+1`: High-contrast glyph, active UI edge, cursor blink, or positive motion delta
//! - **Base-243 Packing (`3^5 = 243` per byte)**: 5 balanced trits pack into 1 byte (`144` trits = `29` bytes).
//! - **PhoeniX `.8a∞` (`0x38 0x61 0xE2 0x9E`) Audio Context**: Time-domain Marine `S + N + T`
//!   (`Sinusoids + Noise + Dirac Transients`) with zero-byte silence deadband.
//! - **Strict 8b-is Exact Arithmetic**: 100% integers, balanced ternary `{-1, 0, +1}`, and reduced `p/q`
//!   rationals (`den > 0`). Zero floating-point types or literals.

use serde::{Deserialize, Serialize};

use crate::screen_ocr::{LlmScreenTranscriptBuffer, transcribe_rgb_console_80x25};

/// `.8a∞` Infinity Audio magic header (`0x38 0x61 0xE2 0x9E`).
pub const EIGHT_A_MAGIC: [u8; 4] = [0x38, 0x61, 0xE2, 0x9E];

/// Ternary grid width (`16` columns for 16:9 HDMI).
pub const TERNARY_GRID_COLS: usize = 16;
/// Ternary grid height (`9` rows for 16:9 HDMI).
pub const TERNARY_GRID_ROWS: usize = 9;
/// Total ternary cells (`16 * 9 = 144`).
pub const TERNARY_GRID_CELLS: usize = TERNARY_GRID_COLS * TERNARY_GRID_ROWS;
/// Base-243 packed byte length for 144 trits (`ceil(144 / 5) = 29` bytes).
pub const TERNARY_PACKED_BYTES: usize = 29;

/// Greatest common divisor (Euclid) for normalizing exact `p/q` rationals.
#[inline]
pub const fn gcd_u64(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    if a == 0 { 1 } else { a }
}

/// Format a signed numerator and positive denominator as a reduced `"p/q"` rational string.
pub fn format_reduced_pq(num: i64, den: u64) -> String {
    let safe_den = if den == 0 { 1 } else { den };
    let abs_num = num.unsigned_abs();
    let g = gcd_u64(abs_num, safe_den);
    let sign = if num < 0 { -1 } else { 1 };
    format!("{}/{}", sign * ((abs_num / g) as i64), safe_den / g)
}

/// Clamp any signed integer into a valid balanced trit `{-1, 0, +1}`.
#[inline]
pub const fn clamp_trit(v: i8) -> i8 {
    if v > 0 {
        1
    } else if v < 0 {
        -1
    } else {
        0
    }
}

/// Pack 5 balanced trits (`{-1, 0, +1}`) into a single Base-243 byte (`0..=242`).
#[inline]
pub const fn pack_5_trits_base243(trits: [i8; 5]) -> u8 {
    let u0 = (clamp_trit(trits[0]) + 1) as u8;
    let u1 = (clamp_trit(trits[1]) + 1) as u8;
    let u2 = (clamp_trit(trits[2]) + 1) as u8;
    let u3 = (clamp_trit(trits[3]) + 1) as u8;
    let u4 = (clamp_trit(trits[4]) + 1) as u8;
    u0 + 3 * u1 + 9 * u2 + 27 * u3 + 81 * u4
}

/// Unpack a Base-243 byte (`0..=242`) back into 5 balanced trits (`{-1, 0, +1}`).
#[inline]
pub const fn unpack_5_trits_base243(byte: u8) -> [i8; 5] {
    let mut v = if byte >= 243 { 242 } else { byte };
    let t0 = (v % 3) as i8 - 1;
    v /= 3;
    let t1 = (v % 3) as i8 - 1;
    v /= 3;
    let t2 = (v % 3) as i8 - 1;
    v /= 3;
    let t3 = (v % 3) as i8 - 1;
    v /= 3;
    let t4 = (v % 3) as i8 - 1;
    [t0, t1, t2, t3, t4]
}

/// Pack all 144 grid trits into 29 Base-243 bytes.
pub fn pack_grid_base243(trits: &[i8; TERNARY_GRID_CELLS]) -> [u8; TERNARY_PACKED_BYTES] {
    let mut out = [0u8; TERNARY_PACKED_BYTES];
    let mut idx = 0usize;
    while idx < TERNARY_PACKED_BYTES {
        let base = idx * 5;
        let mut chunk = [0i8; 5];
        let mut k = 0usize;
        while k < 5 {
            if base + k < TERNARY_GRID_CELLS {
                chunk[k] = trits[base + k];
            }
            k += 1;
        }
        out[idx] = pack_5_trits_base243(chunk);
        idx += 1;
    }
    out
}

/// Unpack 29 Base-243 bytes back into 144 balanced trits (`{-1, 0, +1}`).
pub fn unpack_grid_base243(packed: &[u8; TERNARY_PACKED_BYTES]) -> [i8; TERNARY_GRID_CELLS] {
    let mut out = [0i8; TERNARY_GRID_CELLS];
    let mut idx = 0usize;
    while idx < TERNARY_PACKED_BYTES {
        let chunk = unpack_5_trits_base243(packed[idx]);
        let base = idx * 5;
        let mut k = 0usize;
        while k < 5 {
            if base + k < TERNARY_GRID_CELLS {
                out[base + k] = chunk[k];
            }
            k += 1;
        }
        idx += 1;
    }
    out
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// Semantic screen classification inferred from the `16×9` balanced ternary bitplane.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScreenSemanticClass {
    /// Firmware / UEFI Interactive Shell / BIOS console (glyphs concentrated in upper/left rows over black field).
    UefiOrBiosConsole,
    /// Full-screen text terminal or code workspace (horizontal text bands across screen).
    TerminalOrCodeWorkspace,
    /// Rich graphical desktop or video playback (broad mid-tone + high-contrast multi-quadrant activity).
    GraphicalDesktopOrVideo,
    /// Blank, sleeping, or disconnected HDMI display (predominantly `-1` or `0` trits).
    BlankOrSleepingDisplay,
}

impl ScreenSemanticClass {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::UefiOrBiosConsole => "uefi_or_bios_console",
            Self::TerminalOrCodeWorkspace => "terminal_or_code_workspace",
            Self::GraphicalDesktopOrVideo => "graphical_desktop_or_video",
            Self::BlankOrSleepingDisplay => "blank_or_sleeping_display",
        }
    }
}

/// PhoeniX `.8a∞` (`0x38 0x61 0xE2 0x9E`) time-domain audio context for MCP consumption.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TernaryAudioContext {
    /// `.8a∞` magic header hex (`"3861e29e"`).
    pub eight_a_magic_hex: String,
    /// Dominant thermodynamic state (`"silence_deadband_0x20"`, `"harmonic_tone"`, `"stochastic_ambient"`, `"dirac_transient"`).
    pub thermodynamic_state: String,
    /// Active `.8a` opcode (`0x20` = `LINEUP` silence deadband, `0x30` = `EVT_ATOM` acoustic event).
    pub active_opcode_hex: String,
    /// Sinusoidal (`S`) energy as reduced `p/q` rational (`num/1024`).
    pub s_energy_pq: String,
    /// Stochastic noise (`N`) energy as reduced `p/q` rational (`num/1024`).
    pub n_energy_pq: String,
    /// Dirac transient (`T`) energy as reduced `p/q` rational (`num/1024`).
    pub t_energy_pq: String,
    /// Period jitter (`J_p`) in `p/q` rational units.
    pub period_jitter_pq: String,
    /// Wire bandwidth in bytes/sec (`0` during silence deadband!).
    pub wire_bytes_per_sec: u32,
    /// Balanced ternary acoustic waveform summary (`16` trits in `{-1, 0, +1}`).
    pub waveform_trits: Vec<i8>,
}

impl TernaryAudioContext {
    /// Analyze 16-bit integer PCM samples in the time domain (no FFT, zero floats) into a
    /// PhoeniX `.8a∞` audio context. Empty or sub-threshold input enters `0x20 LINEUP` silence (`0` bytes/sec).
    pub fn from_pcm_i16(samples: &[i16]) -> Self {
        if samples.is_empty() {
            return Self::silence_deadband();
        }

        let mut peak_abs: i64 = 0;
        let mut sum_abs: i64 = 0;
        let mut sum_diff: i64 = 0;
        let mut max_diff: i64 = 0;
        let mut zero_crossings: u64 = 0;
        let mut prev = samples[0] as i64;

        for &s in samples {
            let cur = s as i64;
            let a = cur.abs();
            if a > peak_abs {
                peak_abs = a;
            }
            sum_abs += a;
            let d = (cur - prev).abs();
            sum_diff += d;
            if d > max_diff {
                max_diff = d;
            }
            if (cur >= 0 && prev < 0) || (cur < 0 && prev >= 0) {
                zero_crossings += 1;
            }
            prev = cur;
        }

        let n_len = samples.len() as i64;
        let mean_abs = sum_abs / n_len;
        if mean_abs < 64 && peak_abs < 256 {
            return Self::silence_deadband();
        }

        let s_q10 = ((mean_abs * 1024) / 32768).clamp(1, 1024);
        let n_q10 = (((sum_diff / n_len) * 1024) / 32768).clamp(0, 1024);
        let t_q10 = ((max_diff * 1024) / 65536).clamp(0, 1024);
        let jp_q10 = (((zero_crossings as i64) * 1024) / n_len.max(1)).clamp(0, 1024);

        let state = if t_q10 > 640 {
            "dirac_transient"
        } else if s_q10 >= n_q10 {
            "harmonic_tone"
        } else {
            "stochastic_ambient"
        };

        let mut trits = Vec::with_capacity(16);
        let step = (samples.len() / 16).max(1);
        for i in 0..16 {
            let v = samples.get(i * step).copied().unwrap_or(0) as i64;
            let trit = if v > 512 {
                1
            } else if v < -512 {
                -1
            } else {
                0
            };
            trits.push(trit);
        }

        Self {
            eight_a_magic_hex: bytes_to_hex(&EIGHT_A_MAGIC),
            thermodynamic_state: state.to_string(),
            active_opcode_hex: "0x30_EVT_ATOM".to_string(),
            s_energy_pq: format_reduced_pq(s_q10, 1024),
            n_energy_pq: format_reduced_pq(n_q10, 1024),
            t_energy_pq: format_reduced_pq(t_q10, 1024),
            period_jitter_pq: format_reduced_pq(jp_q10, 1024),
            wire_bytes_per_sec: 168,
            waveform_trits: trits,
        }
    }

    /// Zero-byte `.8a∞` silence deadband (`0x20 LINEUP`).
    pub fn silence_deadband() -> Self {
        Self {
            eight_a_magic_hex: bytes_to_hex(&EIGHT_A_MAGIC),
            thermodynamic_state: "silence_deadband_0x20".to_string(),
            active_opcode_hex: "0x20_LINEUP".to_string(),
            s_energy_pq: "0/1".to_string(),
            n_energy_pq: "0/1".to_string(),
            t_energy_pq: "0/1".to_string(),
            period_jitter_pq: "0/1".to_string(),
            wire_bytes_per_sec: 0,
            waveform_trits: vec![0; 16],
        }
    }
}

/// Complete MCP context packet combining the Balanced Ternary Video Capture Model
/// and PhoeniX `.8a∞` Audio Context.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TernaryMcpContextPacket {
    pub frame_seq: u64,
    pub width: u16,
    pub height: u16,
    pub grid_cols: usize,
    pub grid_rows: usize,
    pub semantic_class: ScreenSemanticClass,
    pub confidence_pq: String,
    pub salience_pq: String,
    pub active_glyph_cells: usize,
    pub steady_surface_cells: usize,
    pub dark_void_cells: usize,
    pub temporal_changed_cells: usize,
    /// Bounding box of `+1` active glyph/UI tiles in normalized grid coordinates (`[x0, y0, x1, y1]`).
    pub active_tile_bbox: [usize; 4],
    /// 29-byte Base-243 packed spatial ternary bitplane (`58` hex chars).
    pub spatial_base243_hex: String,
    /// 29-byte Base-243 packed temporal delta ternary bitplane (`58` hex chars).
    pub temporal_base243_hex: String,
    /// Compact `16×9` visual ternary map (`'█' = +1`, `'▒' = 0`, `'·' = -1`) for direct LLM/MCP reading.
    pub visual_trit_map: Vec<String>,
    /// Live `80×25` console text lines transcribed directly from the HDMI frame (0 floats, 4-bit L2).
    pub transcribed_lines: Vec<String>,
    /// Rolling C64 ring buffer of recent screen state diffs / command outputs for LLM context.
    pub llm_context_buffer: Vec<String>,
    /// Ready-to-feed LLM context digest combining ternary vision, `.8a∞` audio, and transcribed screen text.
    pub llm_prompt_digest: String,
    pub audio: TernaryAudioContext,
}

/// Stateful Ternary Video + Audio Context Analyzer for `RustyNanoKVM`.
#[derive(Clone, Debug)]
pub struct TernaryVideoAnalyzer {
    pub frame_seq: u64,
    pub prev_luma_grid: [u8; TERNARY_GRID_CELLS],
    pub has_prev_frame: bool,
    pub transcript_buffer: LlmScreenTranscriptBuffer,
}

impl Default for TernaryVideoAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl TernaryVideoAnalyzer {
    pub const fn new() -> Self {
        Self {
            frame_seq: 0,
            prev_luma_grid: [0u8; TERNARY_GRID_CELLS],
            has_prev_frame: false,
            transcript_buffer: LlmScreenTranscriptBuffer::new(),
        }
    }

    /// Decode a captured JPEG frame (`jpeg_bytes`) into a `16×9` Balanced Ternary (`{-1, 0, +1}`)
    /// spatial + temporal context packet and pair it with the current PhoeniX `.8a∞` audio context.
    pub fn analyze_jpeg_frame(
        &mut self,
        jpeg_bytes: &[u8],
        pcm_audio: &[i16],
    ) -> TernaryMcpContextPacket {
        self.frame_seq = self.frame_seq.wrapping_add(1);

        let mut mean_luma = [0u8; TERNARY_GRID_CELLS];
        let mut edge_span = [0u8; TERNARY_GRID_CELLS];
        let mut width: u16 = 1920;
        let mut height: u16 = 1080;
        let mut transcribed_lines = Vec::new();

        let sanitized_cow: std::borrow::Cow<'_, [u8]> = if jpeg_bytes.len() > 10
            && jpeg_bytes[0] == 0xFF
            && jpeg_bytes[1] == 0xD8
            && jpeg_bytes[2] == 0xFF
            && jpeg_bytes[3] == 0xE9
            && jpeg_bytes[4] == 0x00
            && jpeg_bytes[5] == 0x04
        {
            let mut v = Vec::with_capacity(jpeg_bytes.len() - 6);
            v.extend_from_slice(&[0xFF, 0xD8]);
            v.extend_from_slice(&jpeg_bytes[8..]);
            std::borrow::Cow::Owned(v)
        } else {
            std::borrow::Cow::Borrowed(jpeg_bytes)
        };

        if let Ok(img) = image::load_from_memory(&sanitized_cow) {
            let rgb = img.to_rgb8();
            let w = rgb.width() as usize;
            let h = rgb.height() as usize;
            width = w.min(65535) as u16;
            height = h.min(65535) as u16;
            transcribed_lines = transcribe_rgb_console_80x25(&rgb);

            if w >= TERNARY_GRID_COLS && h >= TERNARY_GRID_ROWS {
                let tile_w = w / TERNARY_GRID_COLS;
                let tile_h = h / TERNARY_GRID_ROWS;
                // Stride by 4 pixels inside each tile for ultra-fast O(1) embedded RISC-V execution
                let step_x = (tile_w / 16).max(1);
                let step_y = (tile_h / 16).max(1);

                for ty in 0..TERNARY_GRID_ROWS {
                    for tx in 0..TERNARY_GRID_COLS {
                        let cell_idx = ty * TERNARY_GRID_COLS + tx;
                        let y_start = ty * tile_h;
                        let x_start = tx * tile_w;

                        let mut sum_y: u64 = 0;
                        let mut count: u64 = 0;
                        let mut min_y: u8 = 255;
                        let mut max_y: u8 = 0;

                        let mut py = y_start;
                        while py < y_start + tile_h && py < h {
                            let mut px = x_start;
                            while px < x_start + tile_w && px < w {
                                let p = rgb.get_pixel(px as u32, py as u32);
                                // Exact integer ITU-R BT.601 luma: Y = (77*R + 150*G + 29*B) >> 8
                                let y = (((77u32 * (p[0] as u32))
                                    + (150u32 * (p[1] as u32))
                                    + (29u32 * (p[2] as u32)))
                                    >> 8) as u8;
                                sum_y += y as u64;
                                count += 1;
                                if y < min_y {
                                    min_y = y;
                                }
                                if y > max_y {
                                    max_y = y;
                                }
                                px += step_x;
                            }
                            py += step_y;
                        }

                        if count > 0 {
                            mean_luma[cell_idx] = (sum_y / count) as u8;
                            edge_span[cell_idx] = max_y.saturating_sub(min_y);
                        }
                    }
                }
            }
        } else {
            // Fallback bitstream entropy scanner if raw mock/partial bytes are passed
            for (i, chunk) in jpeg_bytes.chunks((jpeg_bytes.len() / TERNARY_GRID_CELLS).max(1)).take(TERNARY_GRID_CELLS).enumerate() {
                let mut hi = 0usize;
                for &b in chunk {
                    if b > 160 {
                        hi += 1;
                    }
                }
                mean_luma[i] = ((hi * 255) / chunk.len().max(1)) as u8;
                edge_span[i] = mean_luma[i];
            }
        }

        let mut spatial_trits = [0i8; TERNARY_GRID_CELLS];
        let mut temporal_trits = [0i8; TERNARY_GRID_CELLS];

        let mut active_glyph_cells = 0usize;
        let mut steady_surface_cells = 0usize;
        let mut dark_void_cells = 0usize;
        let mut temporal_changed_cells = 0usize;

        let mut min_tx = TERNARY_GRID_COLS;
        let mut min_ty = TERNARY_GRID_ROWS;
        let mut max_tx = 0usize;
        let mut max_ty = 0usize;

        let mut top_half_active = 0usize;
        let mut bottom_half_active = 0usize;

        for idx in 0..TERNARY_GRID_CELLS {
            let tx = idx % TERNARY_GRID_COLS;
            let ty = idx / TERNARY_GRID_COLS;
            let m = mean_luma[idx];
            let e = edge_span[idx];

            // Spatial Trit assignment {-1, 0, +1}:
            // +1: High contrast glyph / text / UI border (edge_span >= 48 or bright peak)
            //  0: Steady mid-tone surface (mean_luma >= 22)
            // -1: Dark void background
            let s_trit = if e >= 48 || m >= 140 {
                1i8
            } else if m >= 22 {
                0i8
            } else {
                -1i8
            };
            spatial_trits[idx] = s_trit;

            match s_trit {
                1 => {
                    active_glyph_cells += 1;
                    if tx < min_tx {
                        min_tx = tx;
                    }
                    if ty < min_ty {
                        min_ty = ty;
                    }
                    if tx > max_tx {
                        max_tx = tx;
                    }
                    if ty > max_ty {
                        max_ty = ty;
                    }
                    if ty <= 5 {
                        top_half_active += 1;
                    } else {
                        bottom_half_active += 1;
                    }
                }
                0 => steady_surface_cells += 1,
                _ => dark_void_cells += 1,
            }

            // Temporal Delta Trit assignment {-1, 0, +1}:
            let t_trit = if !self.has_prev_frame {
                clamp_trit(s_trit)
            } else {
                let delta = (m as i16) - (self.prev_luma_grid[idx] as i16);
                if delta > 6 {
                    temporal_changed_cells += 1;
                    1i8
                } else if delta < -6 {
                    temporal_changed_cells += 1;
                    -1i8
                } else {
                    0i8
                }
            };
            temporal_trits[idx] = t_trit;
            self.prev_luma_grid[idx] = m;
        }
        self.has_prev_frame = true;

        let bbox = if active_glyph_cells > 0 {
            [min_tx, min_ty, max_tx, max_ty]
        } else {
            [0, 0, 0, 0]
        };

        // Branchless integer/ternary semantic classification
        let (semantic_class, conf_num) = if active_glyph_cells == 0 && steady_surface_cells < 8 {
            (ScreenSemanticClass::BlankOrSleepingDisplay, 992i64)
        } else if dark_void_cells >= 72 && top_half_active > bottom_half_active * 2 {
            // Classic UEFI Interactive Shell / BIOS boot console signature:
            // high-contrast glyphs concentrated in upper rows over a deep black (-1) field
            (ScreenSemanticClass::UefiOrBiosConsole, 960i64)
        } else if dark_void_cells >= 48 && active_glyph_cells >= 16 {
            (ScreenSemanticClass::TerminalOrCodeWorkspace, 896i64)
        } else {
            (ScreenSemanticClass::GraphicalDesktopOrVideo, 864i64)
        };

        // Relational salience in exact p/q (/1024): combines spatial glyph density + temporal motion
        let raw_salience = ((active_glyph_cells as i64) * 12 + (temporal_changed_cells as i64) * 24 + 256)
            .clamp(64, 1024);

        let spatial_packed = pack_grid_base243(&spatial_trits);
        let temporal_packed = pack_grid_base243(&temporal_trits);

        let mut visual_trit_map = Vec::with_capacity(TERNARY_GRID_ROWS);
        for ty in 0..TERNARY_GRID_ROWS {
            let mut row_str = String::with_capacity(TERNARY_GRID_COLS * 3);
            for tx in 0..TERNARY_GRID_COLS {
                let ch = match spatial_trits[ty * TERNARY_GRID_COLS + tx] {
                    1 => '█',
                    0 => '▒',
                    _ => '·',
                };
                row_str.push(ch);
            }
            visual_trit_map.push(row_str);
        }

        let audio = TernaryAudioContext::from_pcm_i16(pcm_audio);
        let llm_context_buffer = self
            .transcript_buffer
            .ingest_transcribed_lines(self.frame_seq, &transcribed_lines);
        let llm_prompt_digest = format!(
            "[AyeKVM LLM Context | seq={} | class={} | salience={} | active={}/144 | audio={}]\n{}",
            self.frame_seq,
            semantic_class.as_str(),
            format_reduced_pq(raw_salience, 1024),
            active_glyph_cells,
            audio.active_opcode_hex,
            transcribed_lines.join("\n")
        );

        TernaryMcpContextPacket {
            frame_seq: self.frame_seq,
            width,
            height,
            grid_cols: TERNARY_GRID_COLS,
            grid_rows: TERNARY_GRID_ROWS,
            semantic_class,
            confidence_pq: format_reduced_pq(conf_num, 1024),
            salience_pq: format_reduced_pq(raw_salience, 1024),
            active_glyph_cells,
            steady_surface_cells,
            dark_void_cells,
            temporal_changed_cells,
            active_tile_bbox: bbox,
            spatial_base243_hex: bytes_to_hex(&spatial_packed),
            temporal_base243_hex: bytes_to_hex(&temporal_packed),
            visual_trit_map,
            transcribed_lines,
            llm_context_buffer,
            llm_prompt_digest,
            audio,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base243_five_trits_roundtrip_exhaustively() {
        for a in [-1i8, 0, 1] {
            for b in [-1i8, 0, 1] {
                for c in [-1i8, 0, 1] {
                    for d in [-1i8, 0, 1] {
                        for e in [-1i8, 0, 1] {
                            let trits = [a, b, c, d, e];
                            let packed = pack_5_trits_base243(trits);
                            assert!(packed < 243);
                            assert_eq!(unpack_5_trits_base243(packed), trits);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn grid_144_trits_pack_into_29_bytes_and_roundtrip() {
        let mut grid = [0i8; TERNARY_GRID_CELLS];
        for i in 0..TERNARY_GRID_CELLS {
            grid[i] = match i % 3 {
                0 => -1,
                1 => 0,
                _ => 1,
            };
        }
        let packed = pack_grid_base243(&grid);
        assert_eq!(packed.len(), 29);
        let unpacked = unpack_grid_base243(&packed);
        assert_eq!(unpacked, grid);
    }

    #[test]
    fn phoenix_eight_a_audio_silence_deadband_costs_zero_bytes() {
        let silent = TernaryAudioContext::from_pcm_i16(&[0, 1, -1, 0, 2, -2]);
        assert_eq!(silent.active_opcode_hex, "0x20_LINEUP");
        assert_eq!(silent.wire_bytes_per_sec, 0);
        assert_eq!(silent.s_energy_pq, "0/1");
    }
}
