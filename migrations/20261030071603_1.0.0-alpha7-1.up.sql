alter table contests alter column starts_at drop not null;
alter table contests alter column finishes_at drop not null;

create function notify_verdict_change()
returns trigger
language plpgsql
as $$
begin
    if old.verdict is distinct from new.verdict then
        perform pg_notify(
            'submission_verdict_changed',
            new.id::text
        );
    end if;

    return new;
end
$$;

create trigger submission_verdict_trigger
after update of verdict on submissions
for each row
execute function notify_verdict_change();
