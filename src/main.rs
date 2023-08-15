use shuttle_secrets::SecretStore;
use sqlx::PgPool;

mod app_state;
mod commands;
mod db;
pub(crate) mod immut_data;
pub(crate) mod util;
mod bots;
use util::build_client;
use bots::Bot;

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_shared_db::Postgres] pool: PgPool,
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
) -> shuttle_serenity::ShuttleSerenity {
    let bot = Bot::new(pool, secret_store).await;
    let client = build_client(bot).await;
    Ok(client.into())
}

#[cfg(test)]
mod tests {
    use crate::bots::TestBot;

    use super::*;

    // We had to desugar the #[tokio::test] macro because
    // we need to access the secret storage
    #[::core::prelude::v1::test]
    fn test_props() {
        async fn __shuttle_test_props(
            pool: PgPool,
            secret_store: SecretStore,
        ) -> shuttle_serenity::ShuttleSerenity {
            let test_bot = TestBot::new(pool, secret_store).await;
            let client = build_client(test_bot).await;
            Ok(client.into())
        }

        async fn loader(
            mut factory: shuttle_runtime::ProvisionerFactory,
            mut resource_tracker: shuttle_runtime::ResourceTracker,
            logger: shuttle_runtime::Logger,
        ) -> shuttle_serenity::ShuttleSerenity {
            use shuttle_runtime::tracing_subscriber::prelude::*;
            use shuttle_runtime::Context;
            use shuttle_runtime::ResourceBuilder;
            let filter_layer = shuttle_runtime::tracing_subscriber::EnvFilter::try_from_default_env()
                .or_else(|_| shuttle_runtime::tracing_subscriber::EnvFilter::try_new("INFO"))
                .unwrap();
            shuttle_runtime::tracing_subscriber::registry()
                .with(filter_layer)
                .with(logger)
                .init();
            let pool = shuttle_runtime::get_resource(
                shuttle_shared_db::Postgres::new(),
                &mut factory,
                &mut resource_tracker,
            )
            .await
            .context(format!(
                "failed to provision {}",
                stringify!(shuttle_shared_db::Postgres)
            ))?;
            let secret_store = shuttle_runtime::get_resource(
                shuttle_secrets::Secrets::new(),
                &mut factory,
                &mut resource_tracker,
            )
            .await
            .context(format!(
                "failed to provision {}",
                stringify!(shuttle_secrets::Secrets)
            ))?;
            __shuttle_test_props(pool, secret_store).await
        }

        let body = async {
            shuttle_runtime::start(loader).await;
        };
        tokio::pin!(body);
        let body: ::std::pin::Pin<&mut dyn ::std::future::Future<Output = ()>> = body;
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed building the Runtime")
            .block_on(body);
    }
}
