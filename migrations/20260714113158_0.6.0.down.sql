drop table submissions_tests_results;
alter table problems_subgroups
    drop column score_per_test,
    alter column score set not null;
drop index if exists idx_submissions_tests_results_submission_id;
