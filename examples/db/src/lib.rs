pub mod custom;
pub mod schema;

use anyhow::Result;
use testcontainers::{ContainerAsync, ImageExt, runners::AsyncRunner};
use testcontainers_modules::postgres::Postgres;
use tracing::info;

const PG_USER: &str = "postgres";
const PG_PASSWORD: &str = "postgres";
const PG_DATABASE: &str = "postgres";

pub struct TestDb {
    _container: ContainerAsync<Postgres>,
    url: String,
}

impl TestDb {
    pub async fn new() -> Result<Self> {
        info!("Starting database");
        let container = Postgres::default()
            .with_user(PG_USER)
            .with_password(PG_PASSWORD)
            .with_db_name(PG_DATABASE)
            .with_init_sql(include_bytes!("init.sql").to_vec())
            .with_tag("17-alpine")
            .start()
            .await?;

        let url = format!(
            "postgres://{PG_USER}:{PG_PASSWORD}@{}:{}/{PG_DATABASE}",
            container.get_host().await?,
            container.get_host_port_ipv4(5432).await?,
        );

        Ok(Self {
            _container: container,
            url,
        })
    }

    pub fn url(&self) -> &str {
        &self.url
    }
}
