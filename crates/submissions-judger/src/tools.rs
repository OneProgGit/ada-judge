use aj_models::verdicts::TestingVerdict;
use std::{
    env,
    path::{Path, PathBuf},
};
use tools::map::MapLogExt;

pub fn convert_path_in_container_to_path_in_host(path: &Path) -> Result<PathBuf, TestingVerdict> {
    if let Ok(host_run_dir) = env::var("HOST_RUNS_DIR") {
        let host_runs_dir = PathBuf::from(host_run_dir);
        Ok(host_runs_dir.join(
            path.strip_prefix("/")
                .map_log(TestingVerdict::InvalidProblem)?,
        ))
    } else {
        Ok(path.into())
    }
}
