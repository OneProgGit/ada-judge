create table submissions_tests_results (
    submission_id bigint references submissions(id) on delete cascade,
    test int not null,
    test_verdict subgroup_verdict not null,
    score int not null,
    primary key (submission_id, test)
);

alter table problems_subgroups
    add column score_per_test int,
    alter column score drop not null;

create index idx_submissions_tests_results_submission_id on submissions_tests_results (submission_id);
