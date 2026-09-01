CREATE TYPE ticket_status AS ENUM ('todo', 'in_progress', 'done');

CREATE TABLE tickets (
  id          UUID PRIMARY KEY DEFAULT uuidv7(),
  title       TEXT NOT NULL CHECK (char_length(title) BETWEEN 3 AND 20),
  description TEXT NOT NULL CHECK (char_length(description) BETWEEN 5 AND 200),
  status      ticket_status NOT NULL DEFAULT 'todo',
  created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
