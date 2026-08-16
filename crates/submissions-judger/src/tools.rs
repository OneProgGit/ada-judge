use aj_models::verdicts::TestingVerdict;
use std::{
    env,
    path::{Path, PathBuf},
};

pub trait ToHostExt<T> {
    fn to_host(&self) -> Result<PathBuf, TestingVerdict>;
}

impl<T> ToHostExt<T> for T
where
    T: AsRef<Path>,
{
    fn to_host(&self) -> Result<PathBuf, TestingVerdict> {
        let path = self.as_ref();

        if let Ok(host_run_dir) = env::var("HOST_RUNS_DIR") {
            let host_runs_dir = PathBuf::from(host_run_dir);
            Ok(host_runs_dir.join(path.strip_prefix("/").map_err(|_| TestingVerdict::Fail)?))
        } else {
            Ok(path.to_path_buf())
        }
    }
}
