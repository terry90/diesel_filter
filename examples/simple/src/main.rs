#[macro_use]
extern crate diesel_filter;
#[macro_use]
extern crate diesel;

use crate::schema::thingies;
use diesel::prelude::*;
use diesel_derive_newtype::DieselNewType;
use diesel_filter::Paginate;
use std::env;
use uuid::Uuid;

mod schema;

#[derive(Debug, Clone, DieselNewType)]
pub struct CustomType(String);

#[derive(DieselFilter, Queryable, Debug)]
#[diesel(table_name = thingies)]
#[pagination]
// Test `filters_struct_attr` with multiple attributes
#[filters_struct_attr(derive(Default))]
#[filters_struct_attr(derive(Clone))]
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
        custom: Some(CustomType("".into())),
        option_custom: Some(CustomType("".into())),
        ..Default::default()
    };

    // Verify that `#[filters_struct_attr(derive(Clone))]` is applied
    let _ = filters.clone();

    let results = Thingy::filtered(&filters, &mut conn);

    println!("{:?}", filters);
    println!("{:?}", results);
}
