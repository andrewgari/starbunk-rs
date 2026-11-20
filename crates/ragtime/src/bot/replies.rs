//! User-facing copy for the Discord layer (everything not owned by
//! [`crate::presentation`]).

/// Reply to an @mention while a round is already running in the channel.
pub const ALREADY_RUNNING: &str = "A Ragtime round is already running in this channel.";

/// Ephemeral: `/ragtime start` succeeded.
pub const STARTED: &str = "Starting the round!";

/// Ephemeral: the host started with nobody joined; the round was closed.
pub const NO_PLAYERS: &str = "Nobody joined, so the round was closed.";

/// Ephemeral to the host and public notice: the question bank is empty.
pub const NO_QUESTIONS: &str = "There are no questions in the bank yet, so the round was closed.";

/// Ephemeral: loading questions failed.
pub const LOAD_FAILED: &str = "Couldn't load questions. Try again in a moment.";

/// Ephemeral: a non-host tried to start or cancel.
pub const NOT_HOST: &str = "Only the host can do that.";

/// Ephemeral: start/cancel with no round in the channel.
pub const NO_ROUND: &str = "There's no Ragtime round in this channel. @mention me to open one.";

/// Ephemeral: start on a round that is already in progress.
pub const ALREADY_STARTED: &str = "This round has already started.";

/// Ephemeral: `/ragtime cancel` succeeded.
pub const CANCELLED_ACK: &str = "Round cancelled.";

/// Ephemeral modal acknowledgement. Identical for right and wrong answers.
pub const ANSWER_RECEIVED: &str =
    "Answer received! The answer is revealed when the question closes.";

/// Ephemeral: \[Answer\] clicked or modal submitted for a question that has closed.
pub const QUESTION_CLOSED: &str = "That question is already closed.";

/// Public notice: the lobby was never started.
pub const LOBBY_TIMED_OUT: &str = "The Ragtime lobby timed out before it was started.";

/// Public notice: nobody answered for too long.
pub const ABANDONED: &str = "Nobody's answering, so this Ragtime round has ended.";

/// Public notice: the host cancelled.
pub const CANCELLED: &str = "The host cancelled this Ragtime round.";
