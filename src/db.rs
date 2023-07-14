use core::convert::identity as id;
use serenity::model::prelude::UserId;
use sqlx::{FromRow, PgPool};
use ux::u63;

macro_rules! i64_from_as_ref_user_id {
    ($discord_id:expr) => {{
        let UserId(ref discord_id) = $discord_id.as_ref();
        let discord_id: u64 = discord_id.clone();
        let discord_id: i64 = id::<u64>(discord_id) as i64;
        discord_id
    }};
}

/// Data Access Object for [`User`] type.
#[derive(FromRow)]
pub(crate) struct UserDAO {
    pub discord_id: i64,
    pub exp: i64,
}

/// For database operations, [`User`] is converted to [`UserDAO`].
struct User {
    discord_id: u63,
    exp: u63,
}

#[derive(Debug)]
enum FromUserDAOError {
    DiscordId,
    Exp,
}

impl TryFrom<UserDAO> for User {
    type Error = FromUserDAOError;

    fn try_from(UserDAO { discord_id, exp }: UserDAO) -> Result<Self, Self::Error> {
        let Ok(discord_id) = u63::try_from({
            id::<i64>(discord_id) as u64
        }) else {
            return Err(FromUserDAOError::DiscordId);
        };
        let Ok(exp) = u63::try_from({
            id::<i64>(exp) as u64
        }) else {
            return Err(FromUserDAOError::Exp);
        };
        Ok(Self { discord_id, exp })
    }
}

impl From<User> for UserDAO {
    fn from(user: User) -> Self {
        let discord_id: i64 = <u64 as From<u63>>::from(user.discord_id) as i64;
        let exp: i64 = <u64 as From<u63>>::from(user.exp) as i64;
        Self { discord_id, exp }
    }
}

impl UserDAO {
    #[allow(dead_code)]
    pub(crate) async fn query(
        pool: &PgPool,
        UserId(discord_id): UserId,
    ) -> Result<UserDAO, sqlx::Error> {
        let discord_id: i64 = id::<u64>(discord_id) as i64;
        sqlx::query_as("SELECT discord_id, exp FROM app_users WHERE discord_id = $1")
            .bind(discord_id)
            .fetch_one(pool)
            .await
    }

    pub(crate) async fn adjust_exp(
        pool: &PgPool,
        discord_id: impl AsRef<UserId>,
        delta: i64,
    ) -> Result<(), sqlx::Error> {
        let discord_id: i64 = i64_from_as_ref_user_id!(discord_id);
        sqlx::query(
            "INSERT INTO app_users (discord_id, exp) \
        VALUES ($1, $2) \
        ON CONFLICT (discord_id) \
        DO UPDATE SET exp = app_users.exp + $2",
        )
        .bind(discord_id)
        .bind(delta)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub(crate) async fn exp(
        pool: &PgPool,
        discord_id: impl AsRef<UserId>,
    ) -> Result<i64, sqlx::Error> {
        let discord_id = i64_from_as_ref_user_id!(discord_id);
        sqlx::query_scalar("SELECT exp FROM app_users WHERE discord_id = $1")
            .bind(discord_id)
            .fetch_one(pool)
            .await
    }
}
