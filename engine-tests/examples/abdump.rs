//! A/B frames and timings of the cloud fog: `cargo run --release --example abdump -- <out dir> [runs]`.
//! Writes one raw RGBA frame per scene (synthetic world-anchored map: checker ground, unexplored discs and a
//! band) and prints the median update and composite times over `runs` repeats (each repeat = 10 frames with the
//! clock advancing, so the noise is recomputed as in a still map). The same file builds against an older engine
//! copy for before/after comparisons (see DESIGN.md, "0.3.3").

use megamap_engine_tests::engine::clouds::{Clouds, Look, Params, Theme};
use megamap_engine_tests::engine::view::View;
use std::time::Instant;

fn world_image(v: &View, w: usize, h: usize) -> Vec<u8> {
    let mut img = vec![0u8; w * h * 4];
    for y in 0..h {
        for x in 0..w {
            let [wx, wz] = v.to_world(w, h, x as f64, y as f64);
            let i = (y * w + x) * 4;
            let hole = |cx: f64, cz: f64, r: f64| (wx - cx).powi(2) + (wz - cz).powi(2) < r * r;
            if hole(40.0, 10.0, 260.0) || hole(-700.0, 350.0, 180.0) || (wz > 420.0 && wz < 900.0) || wx > 1300.0 {
                continue;
            }
            let c = (((wx / 16.0).floor() + (wz / 16.0).floor()) as i64).rem_euclid(2) as u8;
            img[i..i + 4].copy_from_slice(&[60 + c * 80, 120, 40 + c * 40, 255]);
        }
    }
    img
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let dir = args.get(1).cloned().unwrap_or_else(|| ".".into());
    let runs: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(10);
    let base = Params { lightning: 0, ..Params::default() };
    let looks: Vec<(&str, Params)> = vec![
        ("day_q0", Params { quality: 0, ..base }),
        ("day_q1", Params { quality: 1, ..base }),
        ("day_q2", Params { quality: 2, ..base }),
        ("day_q3", Params { quality: 3, ..base }),
        ("sunset_q3", Params { quality: 3, day_time: 12300, ..base }),
        ("night_q2", Params { quality: 2, day_time: 18000, ..base }),
        ("rain_q2", Params { quality: 2, rain: 1.0, ..base }),
        ("storm_q3", Params { quality: 3, rain: 1.0, thunder: 1.0, theme: Theme::Storm, ..base }),
        ("nether_q2", Params { quality: 2, look: Look::Nether, ..base }),
        ("end_q2", Params { quality: 2, look: Look::End, ..base }),
    ];
    let (w, h) = (1024usize, 576usize);
    for bpp in [0.5f64, 1.0, 2.0, 4.0, 7.0] {
        let mut v = View::new([120.0, 260.0], bpp);
        v.style.mark_unknown = true;
        let img = world_image(&v, w, h);
        for (name, p) in &looks {
            if bpp != 2.0 && !matches!(*name, "day_q1" | "day_q3" | "storm_q3") {
                continue;
            }
            let mut c = Clouds::new();
            let mut out = img.clone();
            c.update(&img, w, h, &v, 7.0, p);
            c.update(&img, w, h, &v, 7.0, p);
            c.composite(&mut out, w, h, p, 7.0);
            std::fs::write(format!("{dir}/{name}_z{bpp}.rgba"), &out).unwrap();
        }
    }
    // Minimap 256 (round).
    let mut vm = View::new([120.0, 260.0], 1.0);
    vm.round = true;
    vm.style.mark_unknown = true;
    let small = world_image(&vm, 256, 256);
    {
        let p = Params { quality: 1, ..base };
        let mut c = Clouds::new();
        let mut out = small.clone();
        c.update(&small, 256, 256, &vm, 7.0, &p);
        c.composite(&mut out, 256, 256, &p, 7.0);
        std::fs::write(format!("{dir}/minimap_q1.rgba"), &out).unwrap();
    }
    // Timings: medians over `runs` repeats.
    let mut v = View::new([120.0, 260.0], 2.0);
    v.style.mark_unknown = true;
    let img = world_image(&v, w, h);
    for q in 0..4u8 {
        let p = Params { quality: q, ..base };
        let (mut us, mut cs) = (Vec::new(), Vec::new());
        let mut c = Clouds::new();
        let mut out = img.clone();
        c.update(&img, w, h, &v, 0.0, &p);
        for r in 0..runs {
            let (mut tu, mut tc) = (0.0, 0.0);
            for i in 0..10 {
                let t = (r * 10 + i) as f32 * 0.05;
                out.copy_from_slice(&img);
                let s = Instant::now();
                c.update(&img, w, h, &v, t, &p);
                tu += s.elapsed().as_secs_f64() * 1e3;
                let s = Instant::now();
                c.composite(&mut out, w, h, &p, t);
                tc += s.elapsed().as_secs_f64() * 1e3;
            }
            us.push(tu / 10.0);
            cs.push(tc / 10.0);
        }
        let (mu, mc) = (median(us), median(cs));
        println!("T 1024x576 q{q}: update {mu:.2} composite {mc:.2} total {:.2} ms", mu + mc);
    }
    let p = Params { quality: 1, ..base };
    let mut c = Clouds::new();
    let mut out = small.clone();
    let mut ts = Vec::new();
    for r in 0..runs {
        let s = Instant::now();
        for i in 0..10 {
            let t = (r * 10 + i) as f32 * 0.05;
            out.copy_from_slice(&small);
            c.update(&small, 256, 256, &vm, t, &p);
            c.composite(&mut out, 256, 256, &p, t);
        }
        ts.push(s.elapsed().as_secs_f64() * 1e3 / 10.0);
    }
    println!("T minimap 256 q1: total {:.2} ms", median(ts));
}
