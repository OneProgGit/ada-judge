drop table submissions_subgroups_results;
drop table submissions;
drop table problems_subgroups;
drop table problems;
drop table contests;
drop table users;

drop type subgroup_type;
drop type total_verdict;
drop type subgroup_verdict;
drop type admin_level;

drop index if exists idx_submissions_contest_user_problem_score;