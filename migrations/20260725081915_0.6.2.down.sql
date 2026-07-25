alter table contests rename column statements_url_ru to statements_url;
alter table contests drop column statements_url_en;
alter table contests rename column editorial_url_ru to editorial_url;
alter table contests drop column editorial_url_en;
