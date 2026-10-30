alter table contests alter column starts_at set not null;
alter table contests alter column finishes_at set not null;

drop function notify_verdict_change();
drop trigger trigger submission_verdict_trigger on submissions;
