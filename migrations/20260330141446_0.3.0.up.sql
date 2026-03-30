create table users (
    id bigserial primary key,
    login text not null,
    password_hash text not null,
    admin_level int not null,
    created_at timestamp not null default now() 
);

create table problems (
    id bigserial primary key,
    owner_id bigint references users(id) on delete cascade,
    name text not null,
    time_limit_ms int not null,
    memory_limit_mb int not null,
    checker_path text not null,
    tests_path text not null,
    created_at timestamp not null default now()
);

insert into problems (owner_id, name, time_limit_ms, memory_limit_mb, checker_path, tests_path)
values
    (null, 'Сумма чисел', 1000, 10, 'checker', 'tests'),
    (null, 'Сумма чисел', 1000, 10, 'checker', 'tests'),
    (null, 'Сумма чисел', 1000, 10, 'checker', 'tests');

create type subgroup_type as enum (
    'sample',
    'main'
);

create table problems_subgroups (
    problem_id bigint references problems(id) on delete cascade,
    subgroup_index bigint not null,
    type text not null,
    tests int[] not null,
    score int not null,
    depends_on int[] not null,
    primary key (problem_id, subgroup_index)
);

insert into problems_subgroups (problem_id, subgroup_index, type, tests, score, depends_on)
values
    (1, 0, 'sample', '{0}', 0, '{}'),
    (1, 1, 'main', '{1,2}', 100, '{}'),
    (2, 0, 'sample', '{0}', 0, '{}'),
    (2, 1, 'main', '{1}', 50, '{0}'),
    (2, 2, 'main', '{2}', 50, '{}'),
    (3, 0, 'sample', '{0}', 0, '{}'),
    (3, 1, 'main', '{1,2}', 100, '{179}');

create type total_verdict as enum (
    'ok',
    'partial_solution',
    'pending',
    'compiling',
    'compilation_error',
    'testing',
    'invalid_problem',
    'invalid_request',
    'bug'
);

create table submissions (
    id bigserial primary key,
    problem_id bigint references problems(id) on delete cascade,
    user_id bigint references users(id) on delete cascade,
    total_verdict total_verdict not null,
    total_score int not null,
    created_at timestamp not null default now()
);

create type subgroup_verdict as enum (
    'ok', 
    'runtime_error', 
    'time_limit_exceeded', 
    'memory_limit_exceeded',
    'security_error',
    'wrong_answer',
    'presentation_error',
    'skipped',
    'testing'
);

create table submissions_subgroups_results (
    submission_id bigint references submissions(id) on delete cascade,
    subgroup_id int not null,
    subgroup_verdict subgroup_verdict not null,
    test int not null,
    score int not null,
    checker_msg text not null,
    primary key (submission_id, subgroup_id)
);