update contests set statements_url = '' where statements_url is null;
alter table contests
alter column statements_url set not null;
