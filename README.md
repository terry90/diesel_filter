# Diesel Filter

Diesel filter is a quick way to add filters and pagination to your diesel models.
Works with `Diesel` and `Postgres`.

## Crate features

- `rocket` Derives `FromForm` on the generated filter struct ([See this example](#with-rocket))
- `actix` Derives `Deserialize` on the generated filter struct ([See this example](#with-actix))
- `pagination` Adds the `Paginate` trait ([See this example](#with-pagination))
- `serialize` with `pagination` Adds the `PaginatedPayload` trait that can directly be sent to your client

## Usage & Examples

Cargo.toml
```toml
diesel_filter = { path = "../../diesel_filter/core", features = ["pagination", "serialize", "rocket"] }
```

Derive your struct with `DieselFilter` and annotate the fields that will be used as filters.
The top level annotation `#[diesel(table_name = db_table)]` is mandatory.

```rust
#[derive(Queryable, DieselFilter)]
#[diesel(table_name = projects)]
pub struct Project {
    pub id: Uuid,
    #[filter(substring, insensitive)]
    pub name: String,
    #[filter(substring)]
    pub owner_email: String,
    #[filter]
    pub owner_id: Uuid,
    pub created_at: NaiveDateTime,
}

```

The `#[filter]` annotation can receive the kinds of filter you want to apply on it, for the moment, there is only `substring` and `insensitive`.

A struct for the filtering data will be generated with the name [YourStructName]Filters, e.g: ProjectFilters.
Two methods will be generated (let's keep `Project` as an example):

```rust
pub fn filter<'a>(filters: &'a ProjectFilters) -> BoxedQuery<'a, Pg>
```

and

```rust
pub fn filtered(filters: &ProjectFilters, conn: &PgConnection) -> Result<Vec<Project>, Error>
```

The `filter` method can be used in conjunction with other diesel methods like `inner_join` and such.

```rust
Project::filter(&filters)
    .inner_join(clients::table)
    .select((projects::id, clients::name))
    .load::<ProjectResponse>(conn)
```

### With Rocket

With the `rocket` feature, the generated struct can be obtained from the request query parameters (dot notation `?filters.name=xxx`)

```rust
use diesel_filter::PaginatedPayload;

#[get("/?<filters>")]
async fn index(filters: ClientFilters, conn: DbConn) -> Result<Json<PaginatedPayload<Client>>, Error> {
    Ok(Json(
        conn.run(move |conn| Client::filtered(&filters, conn))
            .await?
            .into(),
    ))
}

```

### With Actix

With the `actix` feature, the generated struct can be obtained from the request query parameters

N.B: unlike the `rocket` integration, the query parameters must be sent unscopped. e.g `?field=xxx&other=1`

```rust
use diesel_filter::PaginatedPayload;

#[get("/")]
async fn index(filters: web::Query(ClientFilters), conn: DbConn) -> Result<Json<PaginatedPayload<Client>>, Error> {
    Ok(Json(
        conn.run(move |conn| Client::filtered(&filters, conn))
            .await?
            .into(),
    ))
}

```

### With Pagination

With the `pagination` feature, you have access to the methods `paginate`, `per_page` and `load_and_count`

```rust
use diesel_filter::Paginate;

Project::filter(&filters)
    .inner_join(clients::table)
    .select((projects::id, clients::name))
    .paginate(filters.page)
    .per_page(filters.per_page)
    .load_and_count::<ProjectResponse>(conn)
```

These are independent of the `#[pagination]` annotation that you can add on your struct to add `page` and `per_page` to your generated filter struct and change the signature of the `filtered` method.

```rust
#[derive(Queryable, DieselFilter)]
#[diesel(table_name = projects)]
#[pagination]
pub struct Project
```

To convert this into Json, with the feature flag `serialize` you can use `PaginatedPayload`.

```rust
pub struct PaginatedPayload<T> {
    data: Vec<T>,
    total: i64,
}
```

```rust
#[get("/?<filters>")]
async fn index(filters: ProjectFilters, conn: DbConn) -> Result<Json<PaginatedPayload<Project>>, Error> {
    Ok(Json(
        conn.run(move |conn| Project::filtered(&filters))
        .await
        .into(),
    ))
}
```

### `#[filter(multiple)]`

When using `#[filter(multiple)]` with `actix` or `axum` features, parsing of multiple options is done with [`StringWithSeparator<CommaSeparator, T>`](https://docs.rs/serde_with/latest/serde_with/struct.StringWithSeparator.html).

This requires the underlying type to `impl FromStr`, for example:

```rust
#[derive(Debug, DieselNewType)]
pub struct CustomType(String);

impl FromStr for CustomType {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_owned()))
    }
}
```

Enums can use [`EnumString`](https://docs.rs/strum/latest/strum/derive.EnumString.html):

```rust
use strum::EnumString;

#[derive(EnumString, DieselNewType)]
pub enum CustomType {
    A,
    B,
    C,
}

#[derive(DieselFilter)]
struct Model {
    #[filter(multiple)]
    custom: CustomType,
}
```

## License

Diesel filter is licensed under either of the following, at your option:

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
 * MIT License ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)
