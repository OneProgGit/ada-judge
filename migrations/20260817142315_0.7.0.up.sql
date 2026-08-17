begin;

alter table contests rename column ends_at to finishes_at;
alter table contests rename column upsolving_opened to upsolving_enabled;
alter table contests rename column hide_solutions to solutions_hidden;
alter table contests rename column hide_leaderboard to leaderboard_hidden;

create type problem_testing_type as enum (
    'ioi',
    'ioi_merge_subgroups'
);

alter table problems_subgroups alter column score type double precision;
alter table problems_subgroups alter column score_per_test type double precision;
alter type admin_level rename value 'not_admin' to 'user';
update table users set admin_level='user' where admin_level='beta_tester';

create type admin_level_new as enum (
    'not_admin',
    'admin',
    'owner'
);

alter table users
alter column admin_level type admin_level_new
using admin_level::text::admin_level_new;

drop type admin_level;

alter type admin_level_new rename to admin_level;

alter type language rename value 'clang' to 'c';
alter type language rename value 'clangpp' to 'cpp';

alter table submissions alter column total_score rename to score;
alter table submissions alter column total_verdict rename to verdict;
alter table submissions alter column score type double precision;
alter table submissions_subgroups_results alter column score type double precision;
alter table submissions_subgroups_results alter column subgroup_verdict rename to verdict;
alter table submissions_tests_results alter column score type double precision;
alter table submissions_tests_results alter column test_verdict rename to verdict;

update submissions set verdict = 'bug' where verdict in ('invalid_problem', 'invalid_request');
alter type subgroup_verdict rename to verdict;
alter type subgroup_verdict add value 'fail';
alter type total_verdict rename to testing_verdict;
alter type testing_verdict rename value 'bug' to 'fail';

create type testing_verdict_new as enum (
    'ok',
    'partial_solution',
    'pending',
    'compiling',
    'compilation_error',
    'testing',
    'fail'
);

alter table submissions
alter column verdict type testing_verdict_new
using testing_verdict::text::testing_verdict_new;

drop type testing_verdict;

alter type testing_verdict_new rename to testing_verdict;

commit;
