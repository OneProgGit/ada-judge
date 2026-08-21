alter table problems rename column problem_index to index;
alter table problems drop column merge_subgroups;
alter table problems add column testing_type problem_testing_type default 'ioi';
