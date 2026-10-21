create table pages (
  id integer primary key,
  slug text not null,
  lang text not null,
  section text not null default '',
  position integer not null default 0,
  current_rev_id integer,
  deleted_at integer,
  created_at integer not null
);
create unique index pages_slug_lang on pages(slug, lang) where deleted_at is null;

create table revisions (
  id integer primary key,
  page_id integer not null references pages(id),
  body text not null,
  summary text not null default '',
  author_kind text not null check (author_kind in ('user','api_key','import')),
  author_ref text not null default '',
  created_at integer not null
);
create index revisions_page on revisions(page_id, id desc);

create table media (
  id integer primary key,
  filename text not null,
  mime text not null,
  size integer not null,
  sha256 text not null,
  created_at integer not null
);
