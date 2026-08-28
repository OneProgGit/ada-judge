use crate::submissions::{update_subgroup_result, update_submission, update_test_testing_result};
use aj_models::{
    errors::AdaJudgeError,
    testing::{SubgroupResult, TestResult},
    verdicts::{TestingVerdict, Verdict},
};
use sqlx::PgPool;

#[allow(async_fn_in_trait)]
pub trait MapDbExt<T> {
    async fn map_db(
        self,
        pool: &PgPool,
        submission_id: i64,
        subgroup: Option<(i32, i32)>,
    ) -> Result<T, TestingVerdict>;
}

impl<T: Send> MapDbExt<T> for Result<T, TestingVerdict> {
    async fn map_db(self, pool: &PgPool, submission_id: i64, subgroup: Option<(i32, i32)>) -> Self {
        if let Err(verdict) = &self {
            update_submission(pool, submission_id, verdict, 0.)
                .await
                .map_err(|_| TestingVerdict::Fail)?;
            if let Some(subgroup) = subgroup {
                let (subgroup_index, test) = subgroup;
                update_subgroup_result(
                    pool,
                    submission_id,
                    subgroup_index,
                    &SubgroupResult {
                        verdict: Verdict::Fail,
                        test,
                        score: 0.,
                    },
                )
                .await
                .map_err(|_| TestingVerdict::Fail)?;
                update_test_testing_result(
                    pool,
                    submission_id,
                    test,
                    &TestResult {
                        verdict: Verdict::Fail,
                        score: None,
                    },
                )
                .await
                .map_err(|_| TestingVerdict::Fail)?;
            }
        }
        self
    }
}

impl<T: Send> MapDbExt<T> for Result<T, AdaJudgeError> {
    async fn map_db(
        self,
        pool: &PgPool,
        submission_id: i64,
        subgroup: Option<(i32, i32)>,
    ) -> Result<T, TestingVerdict> {
        if self.is_err() {
            update_submission(pool, submission_id, &TestingVerdict::Fail, 0.)
                .await
                .map_err(|_| TestingVerdict::Fail)?;
            if let Some(subgroup) = subgroup {
                let (subgroup_index, test) = subgroup;
                update_subgroup_result(
                    pool,
                    submission_id,
                    subgroup_index,
                    &SubgroupResult {
                        verdict: Verdict::Fail,
                        test,
                        score: 0.,
                    },
                )
                .await
                .map_err(|_| TestingVerdict::Fail)?;
                update_test_testing_result(
                    pool,
                    submission_id,
                    test,
                    &TestResult {
                        verdict: Verdict::Fail,
                        score: None,
                    },
                )
                .await
                .map_err(|_| TestingVerdict::Fail)?;
            }
        }
        self.map_err(|_| TestingVerdict::Fail)
    }
}
