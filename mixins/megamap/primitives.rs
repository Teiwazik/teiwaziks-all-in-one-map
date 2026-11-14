// native_hooks::Game::native_world_primitives (a hook point), HEAD.
// In the world: waypoint beams and labels, the trail.
fn megamap_primitives(game: &crate::game::Game<'_>, camera: [f64; 3], primitives: &mut horizon_client::mod_api::WorldPrimitives) {
    use crate::game::native_megamap as m;
    if game.native_on(m::ID) {
        m::primitives(camera, primitives);
    }
}
