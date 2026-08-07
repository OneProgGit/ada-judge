alter table contests add column hide_leaderboard boolean not null default 'false';

create table contests_co_authors (
    contest_id bigint references contests(id) on delete cascade,
    user_id bigint references users(id) on delete cascade
);
