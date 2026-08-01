alter table contests_posts rename column title_ru to title;
alter table contests_posts rename column text_ru to text;

alter table contests_posts drop column title_en;
alter table contests_posts drop column text_en;
