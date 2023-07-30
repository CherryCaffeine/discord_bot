use crate::util::macros::i64_from_as_ref_user_id;
use serenity::model::prelude::UserId;
use sqlx::PgPool;

pub(crate) mod dao;

pub(crate) async fn add_signed_exp(
    pool: &PgPool,
    discord_id: impl AsRef<UserId>,
    delta: i64,
) -> Result<i64, sqlx::Error> {
    let discord_id: i64 = i64_from_as_ref_user_id!(discord_id);
    sqlx::query_scalar::<_, i64>(
        "INSERT INTO app_users (discord_id, exp) \
    VALUES ($1, $2) \
    ON CONFLICT (discord_id) \
    DO UPDATE SET exp = app_users.exp + $2 \
    RETURNING exp",
    )
    .bind(discord_id)
    .bind(delta)
    .fetch_one(pool)
    .await
}

/// Note that this function returns the active users based on the information
/// *in the database*. They might be on the server anymore
pub(crate) async fn server_members(pool: &PgPool) -> Result<Vec<dao::ServerMember>, sqlx::Error> {
    sqlx::query_as::<_, dao::ServerMember>(
        "SELECT discord_id, exp FROM app_users \
        WHERE on_server = true",
    )
    .fetch_all(pool)
    .await
}

pub(crate) async fn mark_as_quitters(pool: &PgPool, quitters: &[i64]) -> Result<(), sqlx::Error> {
    if quitters.is_empty() {
        return Ok(());
    };
    sqlx::query(
        "UPDATE app_users \
        SET on_server = false \
        WHERE discord_id = ANY($1)",
    )
    .bind(quitters)
    .execute(pool)
    .await?;
    Ok(())
}

pub(crate) async fn add_newcomers(pool: &PgPool, newcomers: &[i64]) -> Result<(), sqlx::Error> {
    if newcomers.is_empty() {
        return Ok(());
    };
    let query = format!(
        "INSERT INTO app_users (discord_id) VALUES {}",
        newcomers
            .iter()
            .enumerate()
            .map(|(i, _)| format!("(${})", i + 1))
            .collect::<Vec<String>>()
            .join(",")
    );

    let mut query_builder = sqlx::query(&query);

    for newcomer in newcomers {
        query_builder = query_builder.bind(newcomer);
    }

    query_builder.execute(pool).await?;
    Ok(())
}

pub(crate) async fn add_earned_role(
    pool: &PgPool,
    role_id: i64,
    exp_needed: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO exp_based_roles (role_id, exp_needed) \
    VALUES ($1, $2) \
    ON CONFLICT (role_id) \
    DO UPDATE SET exp_needed = $2",
    )
    .bind(role_id)
    .bind(exp_needed)
    .execute(pool)
    .await?;
    Ok(())
}
