//! Plan timing on a synthetic workspace with many mod files (ignored; run with
//! `cargo test --release -p fl-core --test bench -- --ignored --nocapture`).
//! `FL_BENCH_FILES` (default 16000) files of `FL_BENCH_KIB` (default 64) KiB each.

mod common;

use std::time::Instant;

use common::*;
use fl_core::import::{Options, import};

fn env(key: &str, default: usize) -> usize {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

#[test]
#[ignore]
fn plan_with_many_mod_files() {
    let (n, kib) = (env("FL_BENCH_FILES", 16_000), env("FL_BENCH_KIB", 64));
    let (_, mut ws) = lab("bench");
    let src = fresh("bench-src").join("Big Pack");
    for i in 0..n {
        let p = src.join(format!("common/d{:03}/f{i:05}.bin", i % 500));
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        let mut bytes = vec![(i % 251) as u8; kib * 1024];
        bytes[..8].copy_from_slice(&(i as u64).to_le_bytes());
        std::fs::write(p, bytes).unwrap();
    }
    let t = Instant::now();
    import(&mut ws, &src, Options::default()).unwrap();
    println!(
        "import: {:?} ({n} files, {} MiB)",
        t.elapsed(),
        n * kib / 1024
    );
    for run in 1..=3 {
        let t = Instant::now();
        let pkgs = ws.packages().unwrap();
        let load = t.elapsed();
        let (plan, _) = ws.apply(&ws.enabled(), "bench", false, true).unwrap();
        println!(
            "run {run}: packages {load:?}, plan {:?} ({} ops, {} files)",
            t.elapsed(),
            plan.ops.len(),
            pkgs["big-pack"].puts.len()
        );
    }
}
