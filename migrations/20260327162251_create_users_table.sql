create table users (
    id bigserial primary key,
    login text not null,
    password_hash text not null,
    admin_level int default 0,
    created_at timestamp default now() 
);