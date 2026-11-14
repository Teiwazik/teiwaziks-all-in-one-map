//! Far-zoom frame benchmark: `cargo run --release --example farbench` in engine-tests.
//! Models what the game pays for a world map frame at a far zoom: the per-frame region planning (`prepare`), the
//! map render over a store of explored and far regions, and the cloud fog, while the map is panned.

use megamap_engine_tests::engine::baker::{bake_chunk, Layer};
use megamap_engine_tests::engine::clouds::{Clouds, Params};
use megamap_engine_tests::engine::region::Region;
use megamap_engine_tests::engine::store::Store;
use megamap_engine_tests::engine::view::{self, Lookup, View};
use megamap_engine_tests::fake::{table, Plains, World};
use std::time::{Duration, Instant};

struct Look<'a> {
    store: &'a Store,
    far: &'a Store,
}
impl Lookup for Look<'_> {
    fn region(&self, rx: i32, rz: i32) -> Option<&Region> {
        self.store.get(rx, rz)
    }
    fn far_region(&self, rx: i32, rz: i32) -> Option<&Region> {
        self.far.get(rx, rz)
    }
    fn has_coarse_far(&self) -> bool {
        false
    }
    fn bounds(&self) -> Option<[i32; 4]> {
        match (self.store.bounds(), self.far.bounds()) {
            (Some(a), Some(b)) => Some([a[0].min(b[0]), a[1].min(b[1]), a[2].max(b[2]), a[3].max(b[3])]),
            (a, b) => a.or(b),
        }
    }
}

fn min_med(v: &mut [Duration]) -> (f64, f64) {
    v.sort();
    (v[0].as_secs_f64() * 1e3, v[v.len() / 2].as_secs_f64() * 1e3)
}

fn main() {
    let w = World { r: 32, cave: false };
    let t = table();
    let mut proto = Region::new(0, 0);
    for cz in 0..32 {
        for cx in 0..32 {
            proto.put_chunk(&bake_chunk(&w, &Plains, &t, cx, cz, Layer::Surface).unwrap());
        }
    }
    // Explored: 5 x 5 regions around the origin; far terrain: 11 x 11 around it.
    let mut store = Store::new(512);
    let mut far = Store::new(512);
    for rz in -5..=5 {
        for rx in -5..=5 {
            let mut r = proto.clone_for_test();
            (r.rx, r.rz) = (rx, rz);
            r.ensure_mips();
            if rx.abs() <= 2 && rz.abs() <= 2 {
                let mut e = proto.clone_for_test();
                (e.rx, e.rz) = (rx, rz);
                e.ensure_mips();
                store.regions.insert((rx, rz), e);
            }
            far.regions.insert((rx, rz), r);
        }
    }
    let lk = Look { store: &store, far: &far };
    let (iw, ih) = (1024usize, 576usize);
    let n = iw * ih * 4;
    let mut base = vec![0u8; n];
    let mut img = vec![0u8; n];
    let frames = 40;
    if let Ok(n) = std::env::var("MM_THREADS") {
        view::set_render_threads(n.parse().unwrap());
    }
    let settle = std::env::var("MM_NOSETTLE").is_err();
    println!("bpp   plan(us)  render min/med   clouds min/med   frame med (ms)");
    for bpp in [2.0f64, 4.0, 8.0, 16.0, 32.0, 64.0, 120.0] {
        let mut v = View::new([100.0, 60.0], bpp);
        v.style = megamap_engine_tests::engine::settings::Settings::default().style(256, true);
        if std::env::var("MM_NOSMOOTH").is_ok() { v.style.smooth = false; }
        if std::env::var("MM_NOTEX").is_ok() { v.style.textures = false; }
        if std::env::var("MM_NOGLOW").is_ok() { v.style.glow = 0; }
        if std::env::var("MM_NORELIEF").is_ok() { v.style.relief = 0; }
        let p = Params { quality: 1, reach: 10.0, ..Params::default() };
        let mut c = Clouds::new();
        let (mut tp, mut tr, mut tc, mut tf) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        let mut vis = Vec::new();
        let (mut tu, mut tu0, mut tk) = (Vec::new(), Vec::new(), Vec::new());
        for i in 0..frames {
            v.center[0] += 7.0 * bpp;
            v.center[1] += 3.0 * bpp;
            let f0 = Instant::now();
            let s = Instant::now();
            Store::plan(&v, iw, ih, &mut vis);
            std::hint::black_box(&vis);
            tp.push(s.elapsed());
            let s = Instant::now();
            view::render(&mut base, iw, ih, &v, &lk);
            if settle {
                view::settle_revealing(&mut base);
            }
            tr.push(s.elapsed());
            let s = Instant::now();
            img.copy_from_slice(&base);
            let s2 = Instant::now();
            let on = c.update(&base, iw, ih, &v, (i / 4) as f32 * 0.1, &p);
            if i % 4 == 0 { tu0.push(s2.elapsed()) } else { tu.push(s2.elapsed()) }
            let s2 = Instant::now();
            if on {
                c.composite(&mut img, iw, ih, &p, (i / 4) as f32 * 0.1);
            }
            tk.push(s2.elapsed());
            tc.push(s.elapsed());
            tf.push(f0.elapsed());
        }
        let (pm, _) = min_med(&mut tp);
        let (r0, r1) = min_med(&mut tr);
        let (c0, c1) = min_med(&mut tc);
        let (_, f1) = min_med(&mut tf);
        println!("      update reuse {:.2} / new t {:.2}, composite {:.2}", min_med(&mut tu).1, min_med(&mut tu0).1, min_med(&mut tk).1);
        println!("{bpp:>5} {:>8.0}  {r0:>6.2} /{r1:>6.2}   {c0:>6.2} /{c1:>6.2}   {f1:>6.2}   [scan {:.2} margin {:.2} dist {:.2} noise {:.2} field {:.2} shade {:.2} fx {:.2}] nodes {} reused {} computed {}", pm * 1e3, c.prof[0], c.prof[1], c.prof[2], c.prof[3], c.prof[4], c.prof[5], c.prof[6], c.last_nodes, c.reused, c.computed);
    }
    // A region read from disk: what the frame paid for its mips before they moved to the worker.
    let mut ts = Vec::new();
    for _ in 0..20 {
        let mut r = proto.clone_for_test();
        let s = Instant::now();
        r.ensure_mips();
        ts.push(s.elapsed());
    }
    let (m0, m1) = min_med(&mut ts);
    println!("full mips of one region: {m0:.2} / {m1:.2} ms");
}
