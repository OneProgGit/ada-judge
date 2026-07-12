alter table contests drop column type;
alter table problems add column type type not null default 'default';
