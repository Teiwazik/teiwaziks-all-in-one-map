// native_hooks::Game::native_screen_mouse_button_released_pre (a hook point), HEAD.
// Mouse up on the world map.
fn megamap_screen_release(game: &crate::game::Game<'_>, button: i32, canceled: &mut bool) {
    use crate::game::native_megamap as m;
    if !*canceled && game.native_on(m::ID) && m::screen_release(game, button) {
        *canceled = true;
    }
}
