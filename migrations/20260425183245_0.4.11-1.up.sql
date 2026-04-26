create type admin_level_new as enum (
    'not_admin',
    'beta_tester',
    'admin',
    'owner'
);

alter table users
set admin_level = 'not_admin';

alter table users
alter column admin_level type admin_level_new
using admin_level::text::admin_level_new;

drop type admin_level;

alter type admin_level_new rename to admin_level;
