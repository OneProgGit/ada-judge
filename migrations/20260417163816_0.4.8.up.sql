create type language as enum (
    'clangpp',
    'clang',
    'go',
    'rust',
    'unknown'
);

alter table submissions add column language language default 'unknown' not null;
alter table contests add column statements_url text default '' not null;
