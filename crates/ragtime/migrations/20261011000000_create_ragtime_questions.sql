-- Ragtime trivia question bank.
CREATE TABLE IF NOT EXISTS ragtime_questions (
    id                 BIGSERIAL PRIMARY KEY,
    question           TEXT        NOT NULL,
    display_answer     TEXT        NOT NULL,
    acceptable_answers TEXT[]      NOT NULL,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT now()
);
