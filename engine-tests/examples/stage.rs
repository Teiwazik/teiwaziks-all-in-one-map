//! Per-stage cloud update profile at 1024x576 (medians of `runs`): `cargo run --release --example stage -- [runs]`.
use megamap_engine_tests::engine::clouds::{Clouds, Params};
use megamap_engine_tests::engine::view::View;

fn main() {
    let runs: usize = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(30);
    let (w, h) = (1024usize, 576usize);
    let mut v = View::new([120.0, 260.0], 2.0);
    v.style.mark_unknown = true;
    let mut img = vec![0u8; w * h * 4];
    for y in 0..h {
        for x in 0..w {
            let [wx, wz] = v.to_world(w, h, x as f64, y as f64);
            let hole = |cx: f64, cz: f64, r: f64| (wx - cx).powi(2) + (wz - cz).powi(2) < r * r;
            if hole(40.0, 10.0, 260.0) || hole(-700.0, 350.0, 180.0) || (wz > 420.0 && wz < 900.0) || wx > 1300.0 {
                continue;
            }
            img[(y * w + x) * 4 + 3] = 255;
        }
    }
    for q in 0..4u8 {
        for reach in [0.0f32, 10.0, 64.0] {
            let p = Params { quality: q, lightning: 0, reach, ..Params::default() };
            let mut c = Clouds::new();
            c.update(&img, w, h, &v, 0.0, &p);
            let mut st: Vec<[f32; 7]> = Vec::new();
            for r in 0..runs {
                c.update(&img, w, h, &v, r as f32 * 0.05, &p);
                st.push(c.prof);
            }
            let med = |k: usize| {
                let mut a: Vec<f32> = st.iter().map(|s| s[k]).collect();
                a.sort_by(|x, y| x.partial_cmp(y).unwrap());
                a[a.len() / 2]
            };
            println!("q{q} reach {reach:>4}: scan {:.2} margin+reach {:.2} dist {:.2} noise {:.2} field {:.2} shade {:.2} | {:.2} ms", med(0), med(1), med(2), med(3), med(4), med(5), med(6));
        }
    }
}
