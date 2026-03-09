create table submissions (
    id bigserial primary key,
    -- TODO: replace with BIGINT
    problem_id text not null,
    user_id bigint not null,
    total_verdict text not null,
    total_score text not null,
    time_stamp timestamp default now()
);

create table submissions_subgroups_results (
    id bigserial primary key,
    submission_id bigint references submissions(id) on delete cascade,
    subgroup_id int not null,
    verdict text not null,
    score text not null,
    checker_msg text
);