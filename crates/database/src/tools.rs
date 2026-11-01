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
            tracing::error!(
                submission_id,
                verdict = %verdict,
                subgroup = ?subgroup,
                "submission step failed"
            );
            update_submission(pool, submission_id, verdict, &String::default(), 0.)
                .await
                .map_err(|error| {
                    tracing::error!(submission_id, %error, "failed to record testing verdict");
                    TestingVerdict::Fail
                })?;
            if let Some(subgroup) = subgroup {
                let (subgroup_index, test) = subgroup;
                update_subgroup_result(
                    pool,
                    submission_id,
                    subgroup_index,
                    &SubgroupResult {
                        verdict: Verdict::Fail,
                        test: Some(test),
                        score: None,
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

impl<T: Send> MapDbExt<T> for Result<T, (TestingVerdict, String)> {
    async fn map_db(
        self,
        pool: &PgPool,
        submission_id: i64,
        subgroup: Option<(i32, i32)>,
    ) -> Result<T, TestingVerdict> {
        match self {
            Err((verdict, compilation_log)) => {
                tracing::error!(
                    submission_id,
                    verdict = %verdict,
                    subgroup = ?subgroup,
                    "submission step failed"
                );
                update_submission(pool, submission_id, &verdict, &compilation_log, 0.)
                    .await
                    .map_err(|error| {
                        tracing::error!(submission_id, %error, "failed to record testing verdict");
                        TestingVerdict::Fail
                    })?;
                if let Some(subgroup) = subgroup {
                    let (subgroup_index, test) = subgroup;
                    update_subgroup_result(
                        pool,
                        submission_id,
                        subgroup_index,
                        &SubgroupResult {
                            verdict: Verdict::Fail,
                            test: Some(test),
                            score: None,
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
                Err(verdict)
            }
            Ok(val) => Ok(val),
        }
    }
}

impl<T: Send> MapDbExt<T> for Result<T, AdaJudgeError> {
    async fn map_db(
        self,
        pool: &PgPool,
        submission_id: i64,
        subgroup: Option<(i32, i32)>,
    ) -> Result<T, TestingVerdict> {
        if let Err(error) = &self {
            tracing::error!(
                submission_id,
                %error,
                subgroup = ?subgroup,
                "database operation failed while testing"
            );
            update_submission(
                pool,
                submission_id,
                &TestingVerdict::Fail,
                &String::default(),
                0.,
            )
            .await
            .map_err(|update_error| {
                tracing::error!(submission_id, %update_error, "failed to record testing verdict");
                TestingVerdict::Fail
            })?;
            if let Some(subgroup) = subgroup {
                let (subgroup_index, test) = subgroup;
                update_subgroup_result(
                    pool,
                    submission_id,
                    subgroup_index,
                    &SubgroupResult {
                        verdict: Verdict::Fail,
                        test: Some(test),
                        score: None,
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
