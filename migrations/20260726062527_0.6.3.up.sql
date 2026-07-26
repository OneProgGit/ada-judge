begin;
alter table problems_subgroups
    alter constraint problems_subgroups_problem_id_fkey
    deferrable initially deferred;
alter table submissions
    alter constraint submissions_problem_id_fkey
    deferrable initially deferred;

set constraints all deferred;

update contests set name_ru = 'Тестовый контест',
    name_en = 'Test contest',
    statements_url_ru = 'https://aj-example.oneprog.org/ru/statements',
    editorial_url_ru = 'https://aj-example.oneprog.org/ru/editorial',
    statements_url_en = 'https://aj-example.oneprog.org/en/statements',
    editorial_url_en = 'https://aj-example.oneprog.org/en/editorial' where id = 1;

update problems set memory_limit_mb = 256 where memory_limit_mb < 256;
update problems set name_en = 'Sum of numbers I' where id = 1;
update problems set name_en = 'Sum of numbers II' where id = 2;
update problems set name_en = 'Sum of numbers III' where id = 3;

update problems set id = id + 100000 where id > 3;
update problems_subgroups set problem_id = problem_id + 100000 where problem_id > 3;
update submissions set problem_id = problem_id + 100000 where problem_id > 3;

insert into problems (id, owner_id, contest_id, problem_index, type, merge_subgroups, name_ru, name_en, time_limit_ms, memory_limit_mb, checker_path, tests_path)
values
    (4, null, 1, 3, 'default', true, 'Сумма чисел IV', 'Sum of numbers IV', 1000, 256, 'checker', 'tests'),
    (5, null, 1, 4, 'default', false, 'Сумма чисел V', 'Sum of numbers IV', 1000, 256, 'checker', 'tests'),
    (6, null, 1, 5, 'interactive', false, 'Угадай число', 'Guess the number', 1000, 256, 'checker', 'tests'),
    (7, null, 1, 6, 'run_twice', true, 'Прибавь 1', 'Add 1', 1000, 256, 'checker', 'tests');

insert into problems_subgroups (problem_id, subgroup_index, type, tests, score, score_per_test, depends_on)
values
    (4, 0, 'sample', '{0}', 0, null, '{}'),
    (4, 1, 'main', '{1}', 50, null, '{0}'),
    (4, 2, 'main', '{2}', 50, null, '{}'),
    (5, 0, 'sample', '{0}', 0, null, '{}'),
    (5, 1, 'main', '{1,2}', null, 50, '{0}'),
    (6, 0, 'sample', '{0}', 0, null, '{}'),
    (6, 1, 'main', '{1,2}', 100, null, '{0}'),
    (7, 0, 'sample', '{0}', 0, null, '{}'),
    (7, 1, 'main', '{1,2}', 100, null, '{0}');

update problems set id = id - 100000 + 4 where id > 100003;
update problems_subgroups set problem_id = problem_id - 100000 + 4 where problem_id > 100003;
update submissions set problem_id = problem_id - 100000 + 4 where problem_id > 100003;

select setval(
    pg_get_serial_sequence('problems', 'id'),
    (select max(id) from problems)
);

commit;
