alter table problems_questions drop column contest_id;
alter table problems_questions add column problem_id bigint references problems(id) on delete cascade;
