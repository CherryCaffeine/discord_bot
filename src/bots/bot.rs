use serenity::{model::prelude::{PartialGuild, Member, Ready, Guild, Message}, async_trait, prelude::{EventHandler, Context, TypeMap}};
use shuttle_secrets::SecretStore;
use sqlx::{PgPool, Executor};
use tokio::sync::RwLockWriteGuard;

use crate::{immut_data::{dynamic::BotConfig, consts::EXP_PER_MSG}, util::members, app_state::{AppState, type_map_keys::{AppStateKey, PgPoolKey}, exp::Exp, self}, commands::Progress};

use super::config_ext::{impl_config_ext, ConfigExt};

pub(crate) struct Bot {
    pub(crate) pool: PgPool,
    pub(crate) bot_config: BotConfig,
}

impl Bot {
    pub(crate) async fn new(pool: PgPool, secret_store: SecretStore) -> Self {
        let bot_config = BotConfig::new(secret_store);
        pool.execute(crate::immut_data::consts::SCHEMA)
            .await
            .expect("Failed to initialize database");
        Self { pool, bot_config }
    }

    fn print_server_members(server: &PartialGuild, members: &[Member]) {
        println!("Members of {} ({} total):", server.name, members.len());

        for m in members.iter() {
            let id = m.user.id;
            let name = m.display_name();
            println!("{id:>20} {name}");
        }
    }
}

impl_config_ext!(Bot);

#[async_trait]
impl EventHandler for Bot {
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

        let res: crate::Result<Exp> = {
            let author: Member = msg.member(&ctx).await.unwrap_or_else(|e| {
                panic!("Failed to get member info for the message author: {e}")
            });
            app_state::sync::add_signed_exp(&ctx.http, &self.bot_config, app_state, &self.pool, &author, EXP_PER_MSG)
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
