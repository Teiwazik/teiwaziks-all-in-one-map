// native_hooks::Game::native_client_level_load (a hook point), HEAD.
// A level (dimension) was loaded: switch the map's layer before the first frame there.
fn megamap_level_load(game: &mut crate::game::Game<'_>, dimension: &str) {
    use crate::game::native_megamap as m;
    if game.native_on(m::ID) {
        m::level_load(game, dimension);
    }
}
