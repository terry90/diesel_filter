use anyhow::Result;
use diesel::prelude::*;
use diesel_async::{AsyncConnection, AsyncPgConnection};
use diesel_filter::DieselFilter;
use diesel_filter_test_db::{TestDb, custom::CustomType, schema::thingies};
use diesel_pagination::{Paginate, PaginationParams};
use serde::{Deserialize, Serialize};
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
    let mut conn = AsyncPgConnection::establish(db.url()).await?;

    let filters_name1 = ThingyFilters {
        name: Some("name1".to_owned()),
        option_num32: Some(1),
        ..Default::default()
    };
    info!(?filters_name1);
    let results_name1 = Thingy::filter(filters_name1)
        .paginate(PaginationParams {
            page: Some(1),
            per_page: None,
        })
        .load_and_count::<Thingy, _>(&mut conn)
        .await?;
    info!(?results_name1);
    assert_eq!(results_name1.items.len(), 1);
    assert_eq!(results_name1.num_total, 1);

    let results_all = Thingy::filter(Default::default())
        .paginate(PaginationParams::default())
        .load_and_count::<Thingy, _>(&mut conn)
        .await?;
    info!(?results_all);
    assert_eq!(results_all.items.len(), 8);
    assert_eq!(results_all.num_total, 8);

    let small_page = Thingy::filter(Default::default())
        .paginate(PaginationParams::page(1).per_page(2))
        .load_and_count::<Thingy, _>(&mut conn)
        .await?;
    info!(?small_page);
    assert_eq!(small_page.items.len(), 2);
    assert_eq!(small_page.num_total, 8);

    Ok(())
}
