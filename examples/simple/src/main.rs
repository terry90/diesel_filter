#[macro_use]
extern crate diesel_filter;
#[macro_use]
extern crate diesel;

use crate::schema::thingies;
use diesel::prelude::*;

mod schema;

#[derive(DieselFilter, Queryable, Debug)]
#[table_name = "thingies"]
struct Thingy {
    pub id: i32,
    #[filter(insensitive)]
    pub name: String,
    #[filter(multiple)]
    pub category: String,
    pub other: String,
}

fn main() {
    // Get a postgres DB connection
    let conn = todo!();

    let mut filters = ThingyFilters {
        name: "coucou",
        category: None,
    };

    let results = ThingyFilters::filtered(&filters, &conn);

    println!("{:?}", filters);
}
