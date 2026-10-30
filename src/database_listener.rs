use crate::app_state::AppState;
use aj_models::contests::ContestsEvent;
use sqlx::{error::BoxDynError, postgres::PgListener};

pub async fn listen_submissions_verdict_changed(state: AppState) -> Result<(), BoxDynError> {
    let mut listener = PgListener::connect_with(&state.db).await?;

    listener.listen("submission_verdict_changed").await?;

    loop {
        let notification = listener.recv().await?;

        tracing::info!(
            payload = notification.payload(),
            "got PostgreSQL notification"
        );

        let submission_id: i64 = notification.payload().parse().map_err(|e| {
            tracing::error!(%e, "failed to parse submission id");
            e
        })?;

        let submission = database::submissions::get_submission(&state.db, submission_id).await?;
        let problem = database::problems::get_problem(&state.db, submission.problem_id).await?;

        state
            .contests_subs
            .get(&Some(problem.contest_id))
            .map(|tx| tx.send(ContestsEvent::SubmissionUpdated(submission_id)));
    }
}
