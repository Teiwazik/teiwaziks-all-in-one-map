// native_hooks::Game::native_screen_char_typed_pre (a hook point), HEAD.
// Typed text on the world map (search box, waypoint name).
fn megamap_screen_char(game: &crate::game::Game<'_>, c: char, canceled: &mut bool) {
    use crate::game::native_megamap as m;
    if !*canceled && game.native_on(m::ID) && m::screen_char(game, c) {
        *canceled = true;
    }
}
