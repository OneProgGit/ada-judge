create type admin_level as enum (
    'not_admin',
    'admin_i',
    'admin_ii',
    'admin_iii',
    'owner'
);

create table users (
    id bigserial primary key,
    login text unique not null,
    password_hash text not null,
    admin_level admin_level default 'not_admin' not null,
    created_at timestamp with time zone not null default now() 
);

create table contests (
    id bigserial primary key,
    owner_id bigint references users(id) on delete cascade,
    name text not null,
    starts_at timestamp with time zone not null,
    ends_at timestamp with time zone not null,
    created_at timestamp with time zone not null default now() 
);

insert into contests (owner_id, name, starts_at, ends_at)
    values (null, 'Суммы чисел', '2001-01-01 01:01:01.000000+00', '20001-01-01 01:01:01.000000+00');

create table problems (
    id bigserial primary key,
    owner_id bigint references users(id) on delete cascade,
    contest_id bigint references contests(id) on delete cascade,
    problem_index bigint not null,
    name text not null,
    time_limit_ms int not null,
    memory_limit_mb int not null,
    checker_path text not null,
    tests_path text not null,
    created_at timestamp with time zone not null default now()
);

insert into problems (owner_id, contest_id, problem_index, name, time_limit_ms, memory_limit_mb, checker_path, tests_path)
values
    (null, 1, 0, 'Сумма чисел I', 1000, 10, 'checker', 'tests'),
    (null, 1, 1, 'Сумма чисел II', 1000, 10, 'checker', 'tests'),
    (null, 1, 2, 'Сумма чисел III', 1000, 10, 'checker', 'tests');

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
    created_at timestamp with time zone not null default now()
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

create index idx_submissions_contest_user_problem_score
    on submissions (user_id, problem_id, total_score desc);