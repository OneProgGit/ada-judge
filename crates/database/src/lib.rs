//! Database tools for `ada-judge`

#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]
#![deny(warnings)]
#![deny(missing_docs)]
#![deny(rustdoc::all)]
#![deny(rustdoc::broken_intra_doc_links)]
#![forbid(unsafe_code)]

use ::tools::map::MapLogExt;
use ada_judge_public_models::verdicts::TotalVerdict;
use models::problems::DatabaseProblemConfig;
use sqlx::PgPool;

pub mod contests;
pub mod submissions;
pub mod tools;
pub mod users;

/// Get's problem's config from `problems` table by given id
/// # Errors
/// Returns an error if `problem_id` is invalid
pub async fn get_problem_by_id(
    pool: &PgPool,
    problem_id: i64,
) -> Result<DatabaseProblemConfig, TotalVerdict> {
    let config = sqlx::query_as(
        "select
                c.id,
                c.owner_id,
                c.contest_id,
                c.problem_index,
                c.name,
                c.time_limit_ms,
                c.memory_limit_mb,
                c.checker_path,
                c.tests_path,
                c.created_at,
                coalesce(
                    json_agg(
                        json_build_object(
                            'type', v.type,
                            'tests', v.tests,
                            'score', v.score,
                            'depends_on', v.depends_on
                        ) order by v.subgroup_index
                    ) filter (where v.problem_id is not null),
                    '[]'
                ) as subgroups
            from problems c
            left join problems_subgroups v on v.problem_id = c.id
            where c.id = $1
            group by c.id,
                c.owner_id,
                c.contest_id,
                c.problem_index,
                c.name,
                c.time_limit_ms,
                c.memory_limit_mb,
                c.checker_path,
                c.tests_path,
                c.created_at
        ",
    )
    .bind(problem_id)
    .fetch_one(pool)
    .await
    .map_log(TotalVerdict::InvalidRequest)?;

    Ok(config)
}
