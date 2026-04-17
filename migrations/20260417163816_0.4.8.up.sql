create type language as enum (
    'clangpp',
    'clang',
    'go',
    'rust',
    'unknown'
);

alter table submissions add column language language default 'unknown';
alter table contests add column statements_url text;
