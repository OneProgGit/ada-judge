create type problem_type as enum (
    'default',
    'interactive',
    'run_twice'
);

alter table contests add column type problem_type not null default 'default';
