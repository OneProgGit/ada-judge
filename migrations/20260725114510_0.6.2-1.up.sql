alter table contests rename column name to name_ru;
alter table contests add column name_en text not null default '';

alter table problems rename column name to name_ru;
alter table problems add column name_en text not null default '';
