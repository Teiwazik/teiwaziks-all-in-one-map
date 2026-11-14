// native_hooks::Game::native_client_disconnecting (a hook point), HEAD.
// Leaving the world or server: write the explored regions and waypoints now.
fn megamap_disconnect(game: &crate::game::Game<'_>) {
    use crate::game::native_megamap as m;
    if game.native_on(m::ID) {
        m::disconnecting();
    }
}
