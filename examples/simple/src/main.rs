#[macro_use]
extern crate diesel_filter;
#[macro_use]
extern crate diesel;

use crate::schema::thingies;
use diesel::prelude::*;
use diesel_filter::Paginate;
use std::env;
use uuid::Uuid;

mod schema;

#[derive(DieselFilter, Queryable, Debug)]
#[diesel(table_name = thingies)]
#[pagination]
pub struct Thingy {
    pub id: Uuid,
    #[filter(insensitive)]
    pub name: String,
    #[filter(multiple)]
    pub category: String,
    #[filter]
    pub other: Option<String>,
}

fn main() {
    // Get a postgres DB connection
    let database_url = env::var("DATABASE_URL").expect("Please set DATABASE_URL");
    let mut conn = PgConnection::establish(&database_url).expect("Could not connect to database");

    let filters = ThingyFilters {
        name: Some("coucou".to_owned()),
        category: None,
        other: None,
        page: None,
        per_page: None,
    };

    let results = Thingy::filtered(&filters, &mut conn);

    println!("{:?}", filters);
    println!("{:?}", results);
}
