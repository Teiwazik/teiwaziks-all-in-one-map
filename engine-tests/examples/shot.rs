//! Writes cloud renders as PPM for eyeballing: `cargo run --release --example shot -- OUT_DIR`.
use megamap_engine_tests::engine::clouds::{Clouds, Params};
use megamap_engine_tests::engine::view::View;

fn main() {
    let out = std::env::args().nth(1).unwrap_or_else(|| ".".into());
    let (w, h) = (768usize, 432usize);
    for (name, bpp, q, stair) in [("q1_b4", 4.0, 1u8, true), ("q3_b4", 4.0, 3, true), ("q3_b1", 1.0, 3, false), ("q2_b05", 0.5, 2, false), ("q0_b16", 16.0, 0, true), ("q1_b3_big", 3.0, 1, true)] {
        let mut img = vec![0u8; w * h * 4];
        if stair {
            for y in 0..h {
                for x in 0..w {
                    let st = if name.ends_with("big") { 40 } else { 16 };
                    if x / st + y / st < 480 / st {
                        let g = if (x / 7 + y / 5) % 3 == 0 { 140 } else { 110 };
                        img[(y * w + x) * 4..][..4].copy_from_slice(&[40, g, 30, 255]);
                    }
                }
            }
        }
        let v = View::new([300.0, -200.0], bpp);
        let p = Params { quality: q, ..Params::default() };
        let mut c = Clouds::new();
        let src = img.clone();
        c.update(&src, w, h, &v, 20.0, &p);
        c.composite(&mut img, w, h, &p, 20.0);
        let mut f = format!("P6 {w} {h} 255\n").into_bytes();
        for px in img.chunks_exact(4) {
            f.extend_from_slice(&px[..3]);
        }
        std::fs::write(format!("{out}/{name}.ppm"), f).unwrap();
    }
}
