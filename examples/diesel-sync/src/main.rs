use anyhow::Result;
use diesel::prelude::*;
use diesel_filter::DieselFilter;
use diesel_filter_test_db::{TestDb, custom::CustomType, schema::thingies};
use serde::{Deserialize, Serialize};
use tokio::task::spawn_blocking;
use tracing::info;
use uuid::Uuid;

#[derive(DieselFilter, Queryable, Debug, Serialize, Deserialize)]
#[diesel(table_name = thingies)]
pub struct Thingy {
    pub id: Uuid,
    #[filter(insensitive)]
    #[serde(skip)]
    pub name: String,
    #[filter]
    pub num32: i32,
    #[filter]
    pub option_num32: Option<i32>,
    #[filter]
    pub num64: i64,
    #[filter]
    pub option_num64: Option<i64>,
    #[filter(multiple, substring, insensitive)]
    pub text: String,
    #[filter]
    pub option_text: Option<String>,
    #[filter]
    pub custom: CustomType,
    #[filter]
    pub option_custom: Option<CustomType>,
    #[filter(multiple)]
    pub multiple_custom: CustomType,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().init();

    let db = TestDb::new().await?;

    spawn_blocking(move || {
        let mut conn = PgConnection::establish(db.url())?;

        let filters_name1 = ThingyFilters {
            name: Some("name1".to_owned()),
            option_num32: Some(1),
            ..Default::default()
        };
        info!(?filters_name1);
        let results_name1 = Thingy::filter(filters_name1).get_results::<Thingy>(&mut conn)?;
        info!(?results_name1);
        assert_eq!(results_name1.len(), 1);

        let filters_nonsense = ThingyFilters {
            name: Some("aaa".to_owned()),
            num64: Some(5),
            ..Default::default()
        };
        info!(?filters_nonsense);
        let results_nonsense = Thingy::filter(filters_nonsense).get_results::<Thingy>(&mut conn)?;
        info!(?results_nonsense);
        assert_eq!(results_nonsense.len(), 0);

        let filters_all = ThingyFilters::default();
        info!(?filters_all);
        let results_all = Thingy::filter(filters_all).get_results::<Thingy>(&mut conn)?;
        info!(?results_all);
        assert_eq!(results_all.len(), 8);

        Ok(())
    })
    .await?
}
