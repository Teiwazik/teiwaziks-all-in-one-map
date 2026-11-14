// native_hooks::Game::native_screen_mouse_scrolled_pre (a hook point), HEAD.
// The wheel on the world map: zoom, or scroll the waypoint list.
fn megamap_screen_scroll(game: &crate::game::Game<'_>, screen: &horizon_client::mod_api::NativeScreenView<'_>, scroll: [f64; 2], canceled: &mut bool) {
    use crate::game::native_megamap as m;
    if !*canceled && game.native_on(m::ID) && m::screen_scroll(game, screen, scroll) {
        *canceled = true;
    }
}
