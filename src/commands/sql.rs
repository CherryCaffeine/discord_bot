use std::collections::HashMap;

use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::Message,
    prelude::Context,
    utils::MessageBuilder,
};
use sqlx::{Column, PgPool, Row, TypeInfo, ValueRef};

use crate::{
    app_state::type_map_keys::{PgPoolKey, BotCfgKey},
    immut_data::dynamic::WHITESPACE,
};

#[command]
#[owners_only]
#[description = "Vampy will run any PostgreSQL <https://www.crunchydata.com/developers/playground/psql-basics> errands for you. Use with caution."]
async fn sql(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let bot_cfg = data
        .get::<BotCfgKey>()
        .unwrap_or_else(|| panic!("Failed to get the bot config from the typemap"));
    let query = {
        let q = msg
            .content
            .trim_start_matches(&bot_cfg.discord_prefix)
            .trim_start_matches("sql ");
        WHITESPACE.replace_all(q, " ")
    };
    println!("Executing query: \"{query}\"");
    let pool: &PgPool = data
        .get::<PgPoolKey>()
        .expect("Failed to get the database pool from the typemap");
    // Result of the query is a vector of rows
    let res: Vec<sqlx::postgres::PgRow> = sqlx::query(&query).fetch_all(pool).await?;
    let mut simplified = Vec::<HashMap<String, String>>::with_capacity(res.len());
    for row in res {
        let columns = row.columns();
        let mut hm = HashMap::<String, String>::with_capacity(columns.len());
        for col in row.columns() {
            let value = row.try_get_raw(col.ordinal()).unwrap();
            let value = match value.format() {
                sqlx::postgres::PgValueFormat::Binary => 'output: {
                    let type_info = value.type_info();
                    let type_name = type_info.name();
                    if value.is_null() {
                        break 'output format!("NULL: ({type_name})");
                    };
                    let slice = match value.as_bytes() {
                        Ok(slice) => slice,
                        Err(e) => break 'output format!("{e:?}: ({type_name})"),
                    };
                    match type_name {
                        "INT8" => {
                            let value = i64::from_be_bytes(slice.try_into().unwrap());
                            format!("{value}: (INT8)")
                        }
                        "BOOL" => {
                            let value: bool = slice[0] == 1;
                            format!("{value:?}: (BOOL)")
                        }
                        "TEXT" => {
                            let value = std::str::from_utf8(slice);
                            format!("{value:?}: (TEXT)")
                        }
                        "VARCHAR" => {
                            let value = std::str::from_utf8(slice);
                            format!("{value:?}: (VARCHAR)")
                        }
                        _ => format!("{slice:?}: ({type_name})"),
                    }
                }
                sqlx::postgres::PgValueFormat::Text => value.as_str().unwrap().to_string(),
            };
            hm.insert(col.name().to_string(), value);
        }
        simplified.push(hm);
    }

    let response = {
        let db_response = serde_json::to_string_pretty(&simplified)?;
        let mut msg_builder = MessageBuilder::new();
        msg_builder
            .mention(&msg.author)
            .push("\n\n")
            .push("Result:\n")
            .push("```json\n")
            .push(&db_response)
            .push("```");
        msg_builder.build()
    };

    msg.reply(&ctx.http, &response).await?;

    Ok(())
}
