// native_hooks::Game::native_frame (a hook point), HEAD.
// Every frame: snapshots for the worker, baked tiles back, the radar and the trail.
fn megamap_frame(game: &mut crate::game::Game<'_>) {
    use crate::game::native_megamap as m;
    if game.native_on(m::ID) {
        m::frame(game);
    }
}
