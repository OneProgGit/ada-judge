create type language_new as enum (
    'clangpp',
    'clang',
    'go',
    'rust',
    'unknown'
);

alter table submissions
alter column language type language_new
using language::text::language_new;

drop type language;

alter type language_new rename to language;
