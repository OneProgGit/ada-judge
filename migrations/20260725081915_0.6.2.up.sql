alter table contests rename column statements_url to statements_url_ru;
alter table contests add column statements_url_en text;
alter table contests rename column editorial_url to editorial_url_ru;
alter table contests add column editorial_url_en text;
