use serenity::{
    async_trait,
    model::prelude::{Guild, Member, Message, PartialGuild, Ready},
    prelude::{Context, EventHandler, TypeMap},
};
use shuttle_secrets::SecretStore;
use sqlx::{Executor, PgPool};
use tokio::sync::RwLockWriteGuard;

use crate::{
    app_state::{
        self,
        exp::Exp,
        type_map_keys::{AppStateKey, PgPoolKey},
        AppState,
    },
    commands::Progress,
    immut_data::{consts::EXP_PER_MSG, dynamic::BotCfg},
    util::members,
};

use super::bot::{impl_bot, Bot};

/// The bot structure that is used to
///
/// * populate the [Context::data] with run-time data during [EventHandler::ready].
/// * handle [EventHandler] events.
///
/// Note that commands do not have the direct access to the [MainBot] struct and
/// use [Context::data] instead.
///
/// The test version of the bot is [`TestBot`](crate::bots::TestBot).
pub(crate) struct MainBot {
    /// Database connection pool for PostgreSQL database.
    /// It is used to persist data between restarts.
    pub(crate) pool: PgPool,
    /// The configuration of the bot.
    pub(crate) cfg: BotCfg,
}

impl MainBot {
    /// Creates a new instance of the bot.
    pub(crate) async fn new(pool: PgPool, secret_store: SecretStore) -> Self {
        let cfg = BotCfg::new(secret_store);
        pool.execute(crate::immut_data::consts::SCHEMA)
            .await
            .expect("Failed to initialize database");
        Self { pool, cfg }
    }

    /// Prints the members of the server to the console.
    fn print_server_members(server: &PartialGuild, members: &[Member]) {
        println!("Members of {} ({} total):", server.name, members.len());

        for m in members.iter() {
            let id = m.user.id;
            let name = m.display_name();
            println!("{id:>20} {name}");
        }
    }
}

impl_bot!(MainBot);

#[async_trait]
impl EventHandler for MainBot {
    async fn ready(&self, ctx: Context, ready: Ready) {
        let members = members(&ctx.http, self.discord_server_id()).await;

        let guild: PartialGuild = Guild::get(&ctx.http, self.discord_server_id()).await
            .unwrap_or_else(|e| panic!("Encountered a Serenity error when getting partial guild information about the discord server: {e:?}"));

        Self::print_server_members(&guild, &members);

        let app_state = AppState::new(&self.pool, members).await;
        {
            let mut wlock: RwLockWriteGuard<TypeMap> = ctx.data.write().await;
            wlock.insert::<AppStateKey>(app_state);
            wlock.insert::<PgPoolKey>(self.pool.clone());
        }

        let bot_name: &str = &ready.user.name;
        println!("{bot_name} is at your service! 🌸");
    }

    async fn message(&self, ctx: Context, msg: Message) {
        let mut wlock = ctx.data.write().await;
        let app_state: &mut AppState = wlock
            .get_mut::<AppStateKey>()
            .expect("Failed to get the app cache from the typemap");
        let AppState {
            users,
            reqd_prompts,
            sorted_earned_roles,
            self_role_msgs: _self_role_msgs,
        } = app_state;
        if let Some((i, req)) = reqd_prompts
            .earned_role
            .iter_mut()
            .enumerate()
            .find(|(_i, req)| req.discord_id == msg.author.id)
        {
            match req
                .progress
                .advance(self, &ctx.http, sorted_earned_roles, users, &msg)
                .await
                .unwrap()
            {
                Some(_req) => (),
                None => {
                    app_state.reqd_prompts.earned_role.remove(i);
                }
            };
            return;
        }
        // we retain wlock because the checks are quick
        if msg.content.starts_with(self.discord_prefix()) {
            return;
        }
        if msg.author.bot {
            return;
        }
        println!("{}: {}", msg.author.name, msg.content);

        let res: crate::util::Result<Exp> = {
            let author: Member = msg.member(&ctx).await.unwrap_or_else(|e| {
                panic!("Failed to get member info for the message author: {e}")
            });
            app_state::sync::add_signed_exp(
                &ctx.http,
                &self.cfg,
                app_state,
                &self.pool,
                &author,
                EXP_PER_MSG,
            )
            .await
        };

        match res {
            Ok(exp) => {
                println!("{}'s exp: {exp:?}", msg.author.name);
            }
            Err(e) => {
                eprintln!("Sqlx error during adjusting experience: {e}");
            }
        };
    }
}
