//! Regenerates `assets/megamap/lang/*.json` from the tables: `cargo run --example lang_json`.
use megamap_engine_tests::engine::lang::{to_json, Lang};

fn main() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../assets/megamap/lang");
    std::fs::write(format!("{dir}/en_us.json"), to_json(Lang::En)).unwrap();
    std::fs::write(format!("{dir}/ru_ru.json"), to_json(Lang::Ru)).unwrap();
}
