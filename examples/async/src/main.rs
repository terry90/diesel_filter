use crate::schema::thingies;
use diesel::prelude::*;
use diesel_async::{
    pooled_connection::{deadpool::{Pool, Object}, AsyncDieselConnectionManager},
    AsyncPgConnection
};
use diesel_derive_newtype::DieselNewType;
use diesel_filter::DieselFilter;
use diesel_filter::Paginate;
use std::env;

use uuid::Uuid;

mod schema;

#[derive(Debug, DieselNewType)]
pub struct CustomType(String);

#[derive(DieselFilter, Queryable, Debug)]
#[diesel(table_name = thingies)]
#[pagination]
pub struct Thingy {
    pub id: Uuid,
    #[filter(insensitive)]
    pub name: String,
    #[filter]
    pub num32: i32,
    #[filter]
    pub option_num32: Option<i32>,
    #[filter]
    pub num64: i64,
    #[filter]
    pub option_num64: Option<i64>,
    #[filter(multiple)]
    pub text: String,
    #[filter]
    pub option_text: Option<String>,
    #[filter]
    pub custom: CustomType,
    #[filter]
    pub option_custom: Option<CustomType>,
}

#[tokio::main]
async fn main() {
    // Get a postgres DB connection
    let database_url = env::var("DATABASE_URL").expect("Please set DATABASE_URL");

    let pool = Pool::builder(AsyncDieselConnectionManager::<AsyncPgConnection>::new(
        database_url,
    ))
    .build()
    .expect("Could not build pool");

    let mut conn = pool.get().await.expect("Unable to get db connection from pool");

    let filters = ThingyFilters {
        name: Some("coucou".to_owned()),
        num32: Some(1),
        option_num32: Some(1),
        num64: Some(1),
        option_num64: Some(1),
        text: None,
        option_text: None,
        custom: Some(CustomType("".into())),
        option_custom: Some(CustomType("".into())),
        page: None,
        per_page: None,
    };

    let results = Thingy::filtered(&filters, &mut conn).await;

    println!("{:?}", filters);
    println!("{:?}", results);
}
