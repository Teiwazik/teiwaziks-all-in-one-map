//! Cloud renders for the detail metric (tools/cloud_detail.py): `cargo run --release --example detail -- OUT_DIR`.
//! An all-unexplored 768x432 image (only the clouds show) at every quality and three zooms.
use megamap_engine_tests::engine::clouds::{Clouds, Params};
use megamap_engine_tests::engine::view::View;

fn main() {
    let out = std::env::args().nth(1).unwrap_or_else(|| ".".into());
    let (w, h) = (768usize, 432usize);
    for bpp in [1.0f64, 2.0, 4.0] {
        for q in 0..4u8 {
            let mut img = vec![0u8; w * h * 4];
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
            std::fs::write(format!("{out}/q{q}_b{bpp}.ppm"), f).unwrap();
        }
    }
}
