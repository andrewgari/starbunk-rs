## [Unreleased] — test/ragtime-presentation-builders

### Added
- `ragtime::presentation`: pure, Discord-I/O-free embed/component/modal builders for the Ragtime
  contest — lobby embed + join emoji constant, question embed + \[Answer\] button + action row,
  answer modal, reveal embed (explicit "nobody answered" case), final tally embed with
  graceful truncation within Discord's embed description limit, and ephemeral
  "not in this round" / "already answered" response text.
- `presentation::custom_id`: `AnswerRoute { channel, index }` encode/parse for both the
  \[Answer\] button and the answer modal `custom_id`, with distinct namespaces so one can't be
  mistaken for the other.
- 38 unit tests covering custom_id round-trips, reveal with zero/one/many correct users, tally
  ordering and ties, and truncation of large leaderboards.
- `wiki/bots/Ragtime.md`: new "Presentation builders" section and updated status table.
