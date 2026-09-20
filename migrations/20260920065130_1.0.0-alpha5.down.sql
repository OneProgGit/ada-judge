alter table submissions
drop constraint submissions_user_id_fkey;

alter table submissions
add constraint submissions_user_id_fkey
foreign key (user_id)
references users(id);
