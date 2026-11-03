alter table media add column data blob not null default x'';
create unique index media_sha256 on media(sha256);
create unique index media_filename on media(filename);
