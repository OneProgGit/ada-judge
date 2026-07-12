alter table contests add column type problem_type not null default 'default';
alter table problems drop column problem_type;
