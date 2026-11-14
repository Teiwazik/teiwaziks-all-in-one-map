// native_hooks::Game::native_commands (a hook point), HEAD.
// The command names megamap runs, for chat completion.
fn megamap_commands(game: &mut crate::game::Game<'_>, commands: &mut Vec<&'static str>) {
    use crate::game::native_megamap as m;
    if game.native_on(m::ID) {
        m::commands(commands);
    }
}
