#[macro_use]
extern crate diesel_filter;
#[macro_use]
extern crate diesel;

use crate::schema::thingies;
use diesel::prelude::*;
use diesel_derive_newtype::DieselNewType;
use diesel_filter::Paginate;
use serde::Deserialize;
use std::{convert::Infallible, env, str::FromStr};
use uuid::Uuid;

mod schema;

#[derive(Debug, Deserialize, DieselNewType)]
pub struct CustomType(String);

impl FromStr for CustomType {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_owned()))
    }
}

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
    #[filter(multiple)]
    pub multiple_custom: CustomType,
}

fn main() {
    // Get a postgres DB connection
    let database_url = env::var("DATABASE_URL").expect("Please set DATABASE_URL");
    let mut conn = PgConnection::establish(&database_url).expect("Could not connect to database");

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
        multiple_custom: Some(vec![CustomType("a".into()), CustomType("b".into())]),
        page: None,
        per_page: None,
    };

    let results = Thingy::filter(filters).get_results(&mut conn).await?;

    println!("{:?}", filters);
    println!("{:?}", results);
}
