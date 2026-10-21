create table users (
  id integer primary key,
  github_id integer not null unique,
  login text not null,
  admin integer not null default 0,
  blocked_at integer,
  created_at integer not null
);

create table api_keys (
  id integer primary key,
  name text not null,
  key_hash text not null unique,
  can_read integer not null default 1,
  can_write integer not null default 0,
  can_export integer not null default 0,
  created_at integer not null,
  last_used_at integer,
  revoked_at integer
);

create table proposals (
  id integer primary key,
  page_id integer not null references pages(id),
  base_rev_id integer not null references revisions(id),
  body text not null,
  summary text not null default '',
  user_id integer not null references users(id),
  status text not null default 'open' check (status in ('open','accepted','rejected')),
  decided_by integer references users(id),
  decided_at integer,
  decision_note text not null default '',
  created_at integer not null
);
create index proposals_status on proposals(status, id);
create index proposals_user on proposals(user_id, created_at);
