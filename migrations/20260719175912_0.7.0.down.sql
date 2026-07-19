create type problem_type_new as enum (
    'default',
    'interactive',
    'run_twice'
);

alter table problems
alter column type type problem_type_new
using problem_type::text::problem_type_new;

drop type problem_type;

alter type problem_type_new rename to problem_type;
