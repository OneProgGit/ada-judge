create table contests_posts (
    id bigserial primary key,
    owner_id bigint references users(id) on delete cascade,
    contest_id bigint references contests(id) on delete cascade,
    title text not null,
    text text not null,
    created_at timestamptz not null default now()
);

create table problems_questions (
    id bigserial primary key,
    owner_id bigint references users(id) on delete cascade,
    contest_id bigint references contests(id) on delete cascade,
    title text not null,
    text text not null,
    answer text not null default '',
    created_at timestamptz not null default now()
);
