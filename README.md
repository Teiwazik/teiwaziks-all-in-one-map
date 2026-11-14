# Teiwazik's All in One Map (`megamap`)

A minimap and fullscreen world map for Horizon with waypoints. It is a native mixin mod (a support module plus hook-point injections), client-side only. The
waypoint features:

- death points
- sharing over chat
- in-world beams with distance labels

Extras:

- a breadcrumb trail
- exploration fog with explored %
- a threat radar
- a cave layer

New in 0.3.2:

- automatic layers: the map switches to a Xaero-style cave slice (ceiling cut, a window of N blocks) when you go
  under ground and back when you see the sky, with a delay so trees and bridges do not flip it and a short
  cross-fade; toolbar button and setting Auto / Surface / Cave / fixed Y
- block colours from the active resource packs (setting "Стиль цвета": vanilla, pack, auto), adjusted when a
  shader pack is on, recomputed when the packs change
- heads instead of dots in the entities window and the settings preview legend; optional anti-aliased icon
  frames from 0.25 to 3 px
- "the interface does not depend on the time of day or weather" (on by default)
- entity, biome and dimension names in the game's language (English / Russian built in), search by name or id
- sliders for auto-zoom near / far and the icon frame

New in 0.3.1:

- far terrain read by 1 to 3 threads at once (setting "Far LOD readers"), about 30 regions a second on the
  spm copy instead of 13, nearest the view first; the log shows the time per phase
- entity faces on the minimap and map (player skins, mob heads from their model) and a 3D preview in the
  entities window; per category: dot, face or 3D icon
- settings in categories with a live preview, clouds kept clear of the map frame, minimap auto-zoom

New in 0.3.0:

- far terrain that really loads: on a server it is read from the chunks Horizon keeps in `server-terrain`,
  nearest first, about 13 regions a second, and saved next to the map (`<dimension>_far`) for the next visit;
  works with the game's LOD rendering on or off
- clouds rewritten: shaped cumulus with self-shadow and silver edges, Nether ash and End haze looks, optional
  lightning and god rays, a Volumetric quality
- the minimap and the map follow the game's frame rate (optional caps); the clouds have their own cap
- a fixed neutral grey bevel frame for the minimap
- world-map toolbar: S (settings in two tabs, info lines reordered with arrows), E (entities: search, categories,
  previews, colour / icon / size, separate minimap and map toggles), D (the saved map of another dimension)

New in 0.2.0:

- far terrain past the explored area from Horizon's LOD data (disk cache, LOD source or live LOD tiles), with or
  without the game's LOD rendering turned on
- exploration fog drawn as animated volumetric cumulus that follow time of day and weather
- map detail: hillshade, cast shadow, ambient occlusion, water depth, shoreline, biome blend, per-block noise,
  plus optional contour lines, chunk / region grid and day / night light
- UI: light / dark / auto theme, frame styles with shadow, compass (N, E, S, W), open animation, a world-map
  toolbar with icons and tooltips, the waypoint list grouped with deaths last

Architecture, performance and API notes are in `DESIGN.md`.

Pinned to Horizon `06ac31ccde2ac174eabdcb0fe82f5109f57f5315`, Mixins `5a64af938ec86b00d53d16a408aabc0038fb3e57`,
game 26.3.

## Install

1. Pack it, or use the files in `dist/` that are already built:
   ```
   python3 tools/gen_art.py      # textures -> assets/megamap/textures/gui/ (optional)
   python3 tools/pack.py         # -> mixins/megamap/runtime.rs, dist/megamap-0.3.11.hnmod (+ .hmod with art)
   cargo-horizon-mod check dist/megamap-0.3.11.hnmod
   ```
   `dist/megamap-0.3.11.hnmod` is the native mod alone (horizon-native.json + mixins/megamap/): put it in
   `horizon-mods/` like the other `.hnmod` files. `dist/megamap-0.3.11.hmod` adds the textures.
2. The launcher builds native mods from source on your machine.
   - **Preferred:** add the `.hnmod` (or the `.hmod`) to the instance's mods in the launcher, turn it on, then
     build the instance's native mods:
     ```
     horizon-launcher native-build --instance <instance> --trust-native-code
     ```
   - **Code only:** building from the folder below skips `assets/`, so the art falls back to drawn shapes:
     ```
     horizon-launcher native-build /home/teiwazik/megamap-mod/mixins/megamap --instance <instance> --trust-native-code
     ```

   Either way this is the long build: it compiles the game with the mod linked in. `--trust-native-code` is
   required because native code is not sandboxed.

## Keys (Options > Mods > megamap to rebind)

| key | action |
|---|---|
| M | world map |
| B | waypoint here (opens the editor) |
| U | waypoint list (search and filter) |
| Z | minimap zoom |
| J | minimap size |
| K | minimap mode: surface, cave or hidden |
| N | rotate with player, or lock north |

## World map

| action | effect |
|---|---|
| drag | pan |
| wheel | zoom around the cursor |
| right-click | menu: waypoint here, coordinates to chat, share position, center here |

The world map also has:

- a status line with the cursor's block coordinates, the zoom and the explored % of the view
- toolbar buttons (icons, tooltip on hover): zoom in and out, center on player, the waypoint list, the cave
  layer, the trail, cloud fog, relief (Flat / Soft / Normal / Strong), grid (Off / Chunks / Regions / Both),
  settings (S), entities (E) and dimension (D)
- a waypoint list of the current dimension, nearest first, with a search box (name filter); click a row to center
  on it, or right-click or use its edit button to open the editor, which also deletes
- an editor for a waypoint's name, colour (8 colours) and icon (diamond, star, home, skull, flag, cross)

## Commands

```
/megamap wp add [name]     waypoint at your position
/megamap wp del <name>
/megamap wp share <name>   posts "[megamap] name @ x y z (dim)" to chat
/megamap wp list
/megamap wp import         adds the last waypoint someone shared in chat
/megamap trail clear
```

## Settings

The megamap page in Options has these settings:

- shape: square or round
- north lock
- corner
- opacity
- info lines: coordinates, biome, time, light
- in-world beams
- radar, including passive mobs
- trail
- fog
- death points kept
- relief shading, contour lines, grid, day / night light
- far terrain (LOD data)
- cloud fog: density, speed, puffiness, theme (auto, day, sunset, night, storm), follow weather, quality
  (Fast, Normal, Fine, Volumetric), FPS limit
- cloud reach: how far the clouds may lie over explored ground, in blocks (0 to 64, default 10), separately for
  the world map and the minimap; the units switch shows it in chunks (steps of 16, rounded down); a 0.3.2 value is
  converted
- UI theme (auto, dark, light), compass, animations
- minimap FPS and map FPS (with the game, or a cap of 144 / 120 / 60 / 30 / 20)
- cloud look (follow map, overworld, Nether, End), Nether and End intensity, lightning and its rate, god rays
- info lines: coordinates, biome, time, light, facing, dimension, in any order (arrows in the map's S panel)
- entity looks per kind (the map's E panel)

The same items are on the world map's S panel, in two tabs. A `settings.txt` from 0.2.0 is read and upgraded.

They are saved in `settings.txt` in the mod's data dir. Region tiles go in `<world>/<dimension>/r.X.Z.mmr` (far tiles in `<world>/<dimension>_far/`) and
waypoints in `<world>/waypoints.txt`.

## Development

```
cd engine-tests
cargo test                              # 27 engine tests, no game needed
cargo run --release --example bench     # bake / encode / compose timings
```

The mod's code is in `src/`: `src/glue.rs` (Horizon side) and `src/engine/` (everything else). Do not edit
`mixins/megamap/runtime.rs`: `tools/pack.py` generates it from `src/`. The hook helpers
(`mixins/megamap/*.rs` besides `runtime.rs`) are hand-written, one per hook point.

Turning megamap off in Options > Mods stops it at once: every helper checks `Game::native_on("megamap")`.

Far terrain works on stock Horizon: the disk cache and the LOD source give full surfaces; live LOD tiles (with
the game's LOD rendering on) use `Lod::ground`, so they lack water and biome tints. For full live tiles, apply
the optional one-function patch to the tree and build with a cfg flag, no repack needed:

```
patch -p1 < patches/far-surface-accessor.patch      # in the staged Horizon tree
RUSTFLAGS="--cfg megamap_far_source" <build>
```

Turn far terrain off in Options > Mods > megamap > "Far terrain (LOD data)". With cloud fog on, far terrain lies
under the fog like unknown ground (it shows where the fog thins and when the fog is off). See DESIGN.md.

## Files

```
horizon-native.json, horizon-mod.toml   manifests (id megamap)
assets/megamap/textures/gui/            generated art
src/glue.rs, src/engine/                the mod's code
mixins/megamap/                         the bundle: horizon-mixin.toml, runtime.rs (generated), hook helpers
engine-tests/                           tests and benchmark
patches/                                proposed Horizon patch
tools/gen_art.py, tools/pack.py
dist/megamap-0.3.11.hnmod, dist/megamap-0.3.11.hmod
```
