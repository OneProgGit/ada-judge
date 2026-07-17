alter table contests add column hide_solutions boolean not null default 'false';

alter table problems add column merge_subgroups boolean not null default 'false';
