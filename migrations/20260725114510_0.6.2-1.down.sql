alter table contests rename column name_ru to name;
alter table contests drop column name_en;

alter table problems rename column name to name_ru;
alter table problems drop column name_en;
