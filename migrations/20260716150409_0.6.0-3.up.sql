alter table contests add column hidden boolean not null default 'false';
alter table contests add column upsolving_opened boolean not null default 'false';
alter table contests add column hide_solutions boolean not null default 'false';

alter table problems add column merge_subgroups boolean not null default 'false';
