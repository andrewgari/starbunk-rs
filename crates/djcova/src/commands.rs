mod clear;
mod help;
mod history;
mod nowplaying;
mod play;
mod queue;
mod repeat;
mod shuffle;
mod skip;
mod stop;
mod volume;

pub use clear::clear_command;
pub use help::help_command;
pub use history::history_command;
pub use nowplaying::nowplaying_command;
pub use play::play_command;
pub use queue::queue_command;
pub use repeat::repeat_command;
pub use shuffle::shuffle_command;
pub use skip::skip_command;
pub use stop::stop_command;
pub use volume::volume_command;

use serenity::all::CreateCommand;

pub fn all_commands() -> Vec<CreateCommand> {
    vec![
        play_command(),
        skip_command(),
        stop_command(),
        queue_command(),
        nowplaying_command(),
        history_command(),
        shuffle_command(),
        help_command(),
        volume_command(),
        clear_command(),
        repeat_command(),
    ]
}
