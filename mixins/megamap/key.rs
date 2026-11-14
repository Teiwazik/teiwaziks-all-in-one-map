// native_hooks::Game::native_key (a hook point), HEAD.
// A key in play: open the map, the waypoint list, a waypoint here; minimap zoom, size, mode, rotation.
fn megamap_key(game: &mut crate::game::Game<'_>, key: &str, pressed: bool) {
    use crate::game::native_megamap as m;
    if game.native_on(m::ID) {
        m::key(game, key, pressed);
    }
}
