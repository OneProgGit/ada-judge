alter table contests add column hidden boolean not null default 'false';
update problems set testing_type = 'ioi_merge_subgroups' where id = 4;
