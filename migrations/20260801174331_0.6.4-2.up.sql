alter table contests_posts rename column title to title_ru;
alter table contests_posts rename column text to text_ru;

alter table contests_posts add column title_en text not null;
alter table contests_posts add column text_en text not null;

update contests_posts set title_en = title_ru;
update contests_posts set text_en = text_ru;
