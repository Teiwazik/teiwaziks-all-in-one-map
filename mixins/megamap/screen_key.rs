// native_hooks::Game::native_screen_key_pressed_pre (a hook point), HEAD.
// Keys on the world map: search, edit fields, the map and list keys.
fn megamap_screen_key(game: &crate::game::Game<'_>, key: &str, canceled: &mut bool) {
    use crate::game::native_megamap as m;
    if !*canceled && game.native_on(m::ID) && m::screen_key(game, key) {
        *canceled = true;
    }
}
