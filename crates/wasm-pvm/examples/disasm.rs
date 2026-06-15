//! Disassemble a JAM/SPI file into one instruction per line for pattern mining.
//!
//! Usage: `cargo run --release --example disasm -- <file.jam>`
//!
//! Output is tab-separated: `byte_offset`, `encoded_len`, then the
//! `Debug`-formatted instruction. Preceded by `# jump_table_entry <offset>`
//! header lines so downstream tooling can reconstruct basic-block leaders.
//!
//! Development tool (used by `experiments/`). Blob-header lengths are narrowed
//! to `usize` with checked `usize::try_from(..).expect(..)` so a malformed file
//! fails loudly rather than silently truncating.
// `ro_len` / `rw_len` are the canonical SPI header field names (ro_data /
// rw_data), used throughout the codebase and the Python analyzers.
#![allow(clippy::similar_names)]

use std::env;
use std::fs;

use wasm_pvm::Instruction;

fn read_var_u32(data: &[u8], off: usize) -> (u64, usize) {
    let fb = u64::from(data[off]);
    if fb < 0x80 {
        (fb, 1)
    } else if fb < 0xc0 {
        (((fb - 0x80) << 8) | u64::from(data[off + 1]), 2)
    } else if fb < 0xe0 {
        (
            ((fb - 0xc0) << 16) | u64::from(data[off + 1]) | (u64::from(data[off + 2]) << 8),
            3,
        )
    } else if fb < 0xf0 {
        (
            ((fb - 0xe0) << 24)
                | u64::from(data[off + 1])
                | (u64::from(data[off + 2]) << 8)
                | (u64::from(data[off + 3]) << 16),
            4,
        )
    } else {
        let mut raw = [0u8; 8];
        raw.copy_from_slice(&data[off + 1..off + 9]);
        (u64::from_le_bytes(raw), 9)
    }
}

/// Bounds-check that `[off, off+need)` lies within `data`, with a clear error
/// for truncated/malformed files instead of a bare index-out-of-bounds panic.
fn need(data: &[u8], off: usize, n: usize, what: &str) {
    let end = off.checked_add(n).expect("offset overflow");
    assert!(
        end <= data.len(),
        "truncated JAM: need {n} bytes for {what} at offset {off}, file has {}",
        data.len(),
    );
}

/// Checked narrowing of a decoded `u64` length to `usize`.
fn usz(v: u64, what: &str) -> usize {
    usize::try_from(v).unwrap_or_else(|_| panic!("{what} ({v}) exceeds usize"))
}

fn main() {
    let path = env::args().nth(1).expect("usage: disasm <file.jam>");
    let data = fs::read(&path).expect("read jam file");

    let mut off = 0usize;
    need(&data, off, 1, "metadata length");
    let (meta_len, n) = read_var_u32(&data, off);
    off += n + usz(meta_len, "metadata length");
    need(&data, off, 3 + 3 + 2 + 3 + 4, "SPI header");
    let ro_len = u32::from_le_bytes([data[off], data[off + 1], data[off + 2], 0]) as usize;
    off += 3;
    let rw_len = u32::from_le_bytes([data[off], data[off + 1], data[off + 2], 0]) as usize;
    off += 3;
    off += 2; // heap pages
    off += 3; // stack size
    need(&data, off, ro_len + rw_len + 4, "ro/rw data + code length");
    off += ro_len + rw_len;
    let code_total_len =
        u32::from_le_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]]) as usize;
    off += 4;
    let blob_end = off.checked_add(code_total_len).expect("blob end overflow");
    assert!(blob_end <= data.len(), "code blob extends past end of file");

    let (jt_len, n) = read_var_u32(&data, off);
    let jt_len = usz(jt_len, "jump-table length");
    off += n;
    let jt_item_bytes = data[off] as usize;
    off += 1;
    // PVM dynamic-jump-table entries are encoded with a uniform width; the blob
    // format only ever uses 0 (empty table) or 4 bytes per entry.
    assert!(
        jt_item_bytes == 0 || jt_item_bytes == 4,
        "unexpected jump-table item width {jt_item_bytes} (expected 0 or 4)"
    );
    let (code_len, n) = read_var_u32(&data, off);
    let code_len = usz(code_len, "code length");
    off += n;

    // Jump table entries are byte offsets of instruction starts.
    let jt_bytes = jt_len
        .checked_mul(jt_item_bytes)
        .expect("jump-table size overflow");
    need(&data, off, jt_bytes, "jump table");
    for i in 0..jt_len {
        let s = off + i * jt_item_bytes;
        let mut v: u64 = 0;
        for (j, b) in data[s..s + jt_item_bytes].iter().enumerate() {
            v |= u64::from(*b) << (8 * j);
        }
        println!("# jump_table_entry {v}");
    }
    off += jt_bytes;
    need(&data, off, code_len, "code section");
    assert!(off + code_len <= blob_end, "code section past blob");

    let code = &data[off..off + code_len];
    let mask = &data[off + code_len..blob_end];

    // Instruction starts from the bitmask.
    let mut starts: Vec<usize> = Vec::new();
    for (byte_idx, byte) in mask.iter().enumerate() {
        for bit in 0..8 {
            let pos = byte_idx * 8 + bit;
            if pos >= code.len() {
                break;
            }
            if (byte >> bit) & 1 == 1 {
                starts.push(pos);
            }
        }
    }
    starts.push(code.len());

    for w in starts.windows(2) {
        let (start, end) = (w[0], w[1]);
        match Instruction::decode(&code[start..end]) {
            Ok((instr, _len)) => {
                println!("{start}\t{}\t{instr:?}", end - start);
            }
            Err(e) => {
                println!("{start}\t{}\tDECODE_ERROR {{ err: \"{e}\" }}", end - start);
            }
        }
    }
}
