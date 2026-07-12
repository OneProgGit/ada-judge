alter table contests drop column type;
alter table problems add column type problem_type not null default 'default';
