pub mod buttons;
mod clear;
mod help;
mod history;
mod nowplaying;
mod play;
mod queue;
mod repeat;
pub mod shared;
mod shuffle;
mod skip;
mod stop;
mod volume;

pub use clear::{clear_command, handle as handle_clear};
pub use help::{handle as handle_help, help_command};
pub use history::{handle as handle_history, history_command};
pub use nowplaying::{handle as handle_nowplaying, nowplaying_command};
pub use play::{handle as handle_play, play_command};
pub use queue::{handle as handle_queue, queue_command};
pub use repeat::{handle as handle_repeat, repeat_command};
pub use shuffle::{handle as handle_shuffle, shuffle_command};
pub use skip::{handle as handle_skip, skip_command};
pub use stop::{handle as handle_stop, stop_command};
pub use volume::{handle as handle_volume, volume_command};

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
