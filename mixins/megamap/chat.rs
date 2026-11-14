// native_hooks::Game::native_client_chat_received (a hook point), HEAD.
// Chat: notice shared waypoints (`[megamap] name x y z dim`). The message is left as it is.
fn megamap_chat(game: &mut crate::game::Game<'_>, message: &mut horizon_client::mod_api::Text) {
    use crate::game::native_megamap as m;
    if game.native_on(m::ID) {
        m::chat(message);
    }
}
