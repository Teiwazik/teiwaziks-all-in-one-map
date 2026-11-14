//! Micro-benchmark: `cargo run --release --example bench` in engine-tests.
//! Bakes a full region of the fake world, encodes/decodes it and renders minimap/world-map views.

use megamap_engine_tests::engine::baker::{bake_chunk, Layer};
use megamap_engine_tests::engine::region::Region;
use megamap_engine_tests::engine::clouds::{Clouds, Look, Params, Theme};
use megamap_engine_tests::engine::view::{self, Lookup, View};
use megamap_engine_tests::fake::{table, Plains, World};
use megamap_engine_tests::engine::baker::ColorTable;
use megamap_engine_tests::engine::layers::{self, Detector, LayerMode, Probe};
use megamap_engine_tests::engine::palette::{self, AverageCache, Sample, Tint};
use std::time::Instant;

struct One(Region);
impl Lookup for One {
    fn region(&self, rx: i32, rz: i32) -> Option<&Region> {
        (rx == 0 && rz == 0).then_some(&self.0)
    }
}

fn main() {
    let w = World { r: 32, cave: false };
    let t = table();
    let mut r = Region::new(0, 0);
    let start = Instant::now();
    let mut n = 0;
    for cz in 0..32 {
        for cx in 0..32 {
            let c = bake_chunk(&w, &Plains, &t, cx, cz, Layer::Surface).unwrap();
            r.put_chunk(&c);
            n += 1;
        }
    }
    let bake = start.elapsed();
    println!("bake: {n} chunks in {:.2?} ({:.1} us/chunk)", bake, bake.as_secs_f64() * 1e6 / n as f64);

    let start = Instant::now();
    let mut bytes = Vec::new();
    for _ in 0..10 {
        bytes = r.encode();
    }
    let enc = start.elapsed() / 10;
    let start = Instant::now();
    for _ in 0..10 {
        std::hint::black_box(Region::decode(&bytes).unwrap());
    }
    let dec = start.elapsed() / 10;
    println!("region: {} bytes (raw 1572864), encode {:.2?}, decode {:.2?}", bytes.len(), enc, dec);

    let start = Instant::now();
    r.ensure_mips();
    println!("mips: {:.2?}", start.elapsed());

    let lk = One(r);
    let mut buf = vec![0u8; 1024 * 1024 * 4];
    for (name, size, rot, bpp) in [("minimap 128 rotated", 128, 0.6, 1.0), ("minimap 256 rotated", 256, 0.6, 1.0), ("world map 1024x576", 1024, 0.0, 0.5)] {
        let h = if size == 1024 { 576 } else { size };
        let v = View { center: [256.0, 256.0], bpp, rotation: rot, round: size != 1024, fog: true, style: Default::default() };
        let start = Instant::now();
        for _ in 0..20 {
            view::render(&mut buf, size, h, &v, &lk);
        }
        println!("render {name}: {:.2?}", start.elapsed() / 20);
    }

    // Relief / contours / night all on: the heaviest map frame.
    let mut v = View::new([256.0, 256.0], 0.5);
    v.style.relief = 3;
    v.style.contours = 8;
    v.style.grid = 3;
    v.style.light = 128;
    let start = Instant::now();
    for _ in 0..20 {
        view::render(&mut buf, 1024, 576, &v, &lk);
    }
    println!("render world map 1024x576, relief 3 + contours + grid + night: {:.2?}", start.elapsed() / 20);

    // Cloud frame: a world map that is mostly unexplored (region 0,0 in a corner).
    for (bpp, label) in [(2.0, "mostly fog"), (0.5, "half fog")] {
        let mut v = View::new([900.0, 700.0], bpp);
        v.style.mark_unknown = true;
        view::render(&mut buf, 1024, 576, &v, &lk);
        let base = buf[..1024 * 576 * 4].to_vec();
        let looks = [("day", Params::default()), ("nether", Params { look: Look::Nether, ..Params::default() }), ("end", Params { look: Look::End, ..Params::default() })];
        for (q, (look, lp)) in (0..4u8).flat_map(|q| looks.iter().map(move |l| (q, l))) {
            if *look != "day" && q != 1 {
                continue;
            }
            let p = Params { quality: q, ..*lp };
            let mut c = Clouds::new();
            let mut img = base.clone();
            c.update(&base, 1024, 576, &v, 0.0, &p);
            let n = 20;
            let (mut tu, mut tc) = (std::time::Duration::ZERO, std::time::Duration::ZERO);
            let mut prof = [0f32; 7];
            for i in 0..n {
                img.copy_from_slice(&base);
                let s = Instant::now();
                c.update(&base, 1024, 576, &v, i as f32 * 0.05, &p);
                tu += s.elapsed();
                for k in 0..7 {
                    prof[k] += c.prof[k] / n as f32;
                }
                let s = Instant::now();
                c.composite(&mut img, 1024, 576, &p, i as f32 * 0.05);
                tc += s.elapsed();
            }
            println!(
                "clouds 1024x576 {label}, {look}, quality {q}: nodes {} update {:.2?} composite {:.2?} [scan {:.2} margin {:.2} dist {:.2} noise {:.2} field {:.2} shade {:.2} fx {:.2} ms]",
                c.last_nodes,
                tu / n,
                tc / n,
                prof[0], prof[1], prof[2], prof[3], prof[4], prof[5], prof[6]
            );
        }
        let p = Params { quality: 1, ..Params::default() };
        let mut c = Clouds::new();
        let mm = &base[..0];
        let _ = mm;
        let mut v = View::new([900.0, 700.0], 1.0);
        v.round = true;
        v.style.mark_unknown = true;
        let mut small = vec![0u8; 256 * 256 * 4];
        view::render(&mut small, 256, 256, &v, &lk);
        let sb = small.clone();
        let s = Instant::now();
        for i in 0..20 {
            small.copy_from_slice(&sb);
            c.update(&sb, 256, 256, &v, i as f32 * 0.05, &p);
            c.composite(&mut small, 256, 256, &p, i as f32 * 0.05);
        }
        println!("clouds minimap 256x256 ({label} pass), quality 1: {:.2?} [scan {:.2} margin {:.2} dist {:.2} noise {:.2} field {:.2} shade {:.2} ms, grid {}x{}, distance domain {}x{}]", s.elapsed() / 20, c.prof[0], c.prof[1], c.prof[2], c.prof[3], c.prof[4], c.prof[5], c.dims()[0], c.dims()[1], c.dims()[2], c.dims()[3]);
    }

    // Auto layers: what the frame pays (the probe, the slice pick, the cross-fade of a switch) and a cave slice bake.
    {
        let mut d = Detector::default();
        let n = 1_000_000;
        let s = Instant::now();
        let mut acc = 0i64;
        for i in 0..n {
            let y = 40 + (i % 64) as i32;
            let p = Probe { y, sky: Some((i % 16) as u8), ground: Some(70) };
            d.update(i as f32 * 0.016, p, 0);
            acc += layers::slice(LayerMode::Auto, d.cave, y, 0).unwrap_or(0) as i64;
        }
        std::hint::black_box(acc);
        println!("auto layers: probe + slice {:.1} ns/frame", s.elapsed().as_secs_f64() * 1e9 / n as f64);
        let mut img = vec![200u8; 256 * 256 * 4];
        let old = vec![20u8; 256 * 256 * 4];
        let s = Instant::now();
        for i in 0..200 {
            layers::crossfade(&mut img, &old, (i % 10) as f32 / 10.0);
        }
        println!("auto layers: cross-fade 256x256 {:.2?}/frame (only for {} s after a switch)", s.elapsed() / 200, layers::FADE_S);
        let cw = World { r: 32, cave: true };
        let (top, floor) = layers::cave_window(layers::bucket_top(40), 32);
        let s = Instant::now();
        let mut nc = 0;
        for cz in 0..8 {
            for cx in 0..8 {
                std::hint::black_box(bake_chunk(&cw, &Plains, &t, cx, cz, Layer::Cave(top, floor)));
                nc += 1;
            }
        }
        println!("auto layers: cave slice bake {:.1} us/chunk (worker thread)", s.elapsed().as_secs_f64() * 1e6 / nc as f64);
        let s = Instant::now();
        for _ in 0..20 {
            view::render(&mut buf, 256, 256, &View { center: [256.0, 256.0], bpp: 1.0, rotation: 0.6, round: true, fog: true, style: Default::default() }, &lk);
        }
        println!("auto layers: minimap 256 frame in cave mode {:.2?}", s.elapsed() / 20);
    }

    // Resource-pack palette: 24000 block states over 2400 distinct 16x16 textures (a big pack), decoded once each.
    {
        let states = 24_000u32;
        let mut vanilla = ColorTable::with_len(states as usize);
        for id in 1..states {
            vanilla.set(id, (id % 60) as u8 + 1, 0x808080);
        }
        let textures: Vec<Vec<u8>> = (0..2400u32).map(|k| (0..16 * 16 * 4).map(|i| ((i as u32 * 31 + k * 17) % 251) as u8).collect()).collect();
        let s = Instant::now();
        let mut cache = AverageCache::default();
        let mut samples = Vec::with_capacity(states as usize);
        for id in 1..states {
            let k = (id % 2400) as usize;
            let name = format!("block/t{k}");
            if let Some(rgb) = cache.get(&name, |_| Some((16, 16, textures[k].clone()))) {
                let tint = match id % 7 { 0 => Tint::Grass, 1 => Tint::Foliage, _ => Tint::None };
                samples.push(Sample { id, rgb, tint });
            }
        }
        let sampled = s.elapsed();
        let s = Instant::now();
        let table = palette::override_table(&vanilla, &samples, true);
        let over = s.elapsed();
        std::hint::black_box(table.len());
        let s = Instant::now();
        for _ in 0..1000 {
            std::hint::black_box(palette::pack_hash(42, &["file/pack_a.zip".into(), "file/pack_b".into()], "BSL", 2));
        }
        println!(
            "palette: {} states, {} textures decoded, sample {:.2?} + table {:.2?} (once per pack change, worker thread); pack_hash {:.0} ns",
            samples.len(),
            cache.decoded,
            sampled,
            over,
            s.elapsed().as_secs_f64() * 1e9 / 1000.0
        );
    }

    // Drag frame: the map is panned 7 px per frame for 60 frames (base map re-rendered every frame, as the game
    // does on a view change). "held t" = the glue's DRAG_TICK (cloud time steps every 2nd frame at 60 fps),
    // "running t" = the time advances every frame (cloud_fps 0 and slow frames).
    for q in 0..4u8 {
        let p = Params { quality: q, ..Params::default() };
        for (mode, step) in [("held t", 2usize), ("running t", 1)] {
            let mut c = Clouds::new();
            let mut v = View::new([900.0, 700.0], 2.0);
            v.style.mark_unknown = true;
            let mut base = vec![0u8; 1024 * 576 * 4];
            view::render(&mut base, 1024, 576, &v, &lk);
            let mut img = base.clone();
            c.update(&base, 1024, 576, &v, 0.0, &p);
            let (mut tr, mut tu, mut tc, mut reused, mut computed) = (std::time::Duration::ZERO, std::time::Duration::ZERO, std::time::Duration::ZERO, 0, 0);
            let n = 60;
            for i in 0..n {
                v.center[0] += 7.0 * v.bpp;
                v.center[1] += 3.0 * v.bpp;
                let s = Instant::now();
                view::render(&mut base, 1024, 576, &v, &lk);
                tr += s.elapsed();
                let t = (i / step) as f32 * 0.033;
                img.copy_from_slice(&base);
                let s = Instant::now();
                c.update(&base, 1024, 576, &v, t, &p);
                tu += s.elapsed();
                let s = Instant::now();
                c.composite(&mut img, 1024, 576, &p, t);
                tc += s.elapsed();
                reused += c.reused;
                computed += c.computed;
            }
            println!(
                "drag 1024x576 quality {q} {mode}: map {:.2?} clouds update {:.2?} composite {:.2?} (noise nodes reused {} computed {} per frame)",
                tr / n as u32,
                tu / n as u32,
                tc / n as u32,
                reused / n,
                computed / n
            );
        }
    }

    // MM_DUMP=<dir>: raw RGBA frames of every look for eyeballing (converted to PNG outside).
    if let Ok(dir) = std::env::var("MM_DUMP") {
        let mut v = View::new([900.0, 700.0], 1.0);
        v.style.mark_unknown = true;
        view::render(&mut buf, 1024, 576, &v, &lk);
        let base = buf[..1024 * 576 * 4].to_vec();
        let cases = [
            ("day_q1", Params { quality: 1, ..Params::default() }),
            ("day_q3", Params { quality: 3, ..Params::default() }),
            ("sunset_q3", Params { quality: 3, day_time: 12300, ..Params::default() }),
            ("night_q3", Params { quality: 3, day_time: 18000, ..Params::default() }),
            ("storm_q2", Params { quality: 2, theme: Theme::Storm, ..Params::default() }),
            ("nether_q2", Params { quality: 2, look: Look::Nether, ..Params::default() }),
            ("end_q2", Params { quality: 2, look: Look::End, ..Params::default() }),
        ];
        for (name, p) in cases {
            let mut c = Clouds::new();
            let mut img = base.clone();
            c.update(&base, 1024, 576, &v, 7.0, &p);
            c.composite(&mut img, 1024, 576, &p, 7.0);
            std::fs::write(format!("{dir}/{name}.rgba"), &img).unwrap();
        }
        // Zoomed out: aliasing check.
        let mut v = View::new([900.0, 700.0], 8.0);
        v.style.mark_unknown = true;
        view::render(&mut buf, 1024, 576, &v, &lk);
        let base = buf[..1024 * 576 * 4].to_vec();
        let p = Params { quality: 1, ..Params::default() };
        let mut c = Clouds::new();
        let mut img = base.clone();
        c.update(&base, 1024, 576, &v, 7.0, &p);
        c.composite(&mut img, 1024, 576, &p, 7.0);
        std::fs::write(format!("{dir}/far_q1.rgba"), &img).unwrap();
    }
}
