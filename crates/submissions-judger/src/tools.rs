use models::verdicts::TotalVerdict;
use sqlx::PgPool;
use std::{
    env,
    path::{Path, PathBuf},
};
use tools::map::MapLogExt;

use database::update_total_testing_result;

pub(crate) fn convert_path_in_container_to_path_in_host(
    path: &Path,
) -> Result<PathBuf, TotalVerdict> {
    if let Ok(host_run_dir) = env::var("HOST_RUNS_DIR") {
        let host_runs_dir = PathBuf::from(host_run_dir);
        Ok(host_runs_dir.join(
            path.strip_prefix("/")
                .map_log(TotalVerdict::InvalidProblem)?,
        ))
    } else {
        Ok(path.into())
    }
}

pub(crate) trait MapDbExt<T> {
    async fn map_db(self, pool: &PgPool, submission_id: i64) -> Result<T, TotalVerdict>;
}

impl<T: Send> MapDbExt<T> for Result<T, TotalVerdict> {
    async fn map_db(self, pool: &PgPool, submission_id: i64) -> Result<T, TotalVerdict> {
        if let Err(verdict) = &self {
            log::error!("Error verdict: {verdict}");
            update_total_testing_result(pool, submission_id, verdict, 0).await?;
        }
        self
    }
}
