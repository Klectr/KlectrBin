create table if not exists pastes (
  id serial primary key,
  title text NOT NULL,
  content text NOT NULL
);
