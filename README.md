# `diesel-filter`

Diesel filter is a quick way to add filters and pagination to your diesel models.
Works with `diesel` and `Postgres`.

## Crate features

- `rocket` Derives `FromForm` on the generated filter struct ([See this example](#with-rocket))
- `actix` Derives `Deserialize` on the generated filter struct ([See this example](#with-actix))
- `serialize` with `pagination` Adds the `PaginatedPayload` trait that can directly be sent to your client

## Changes in 2.0

* Pagination was moved to a new crate called `diesel-pagination`, see new usage below.

## Usage & Examples

Cargo.toml

```toml
diesel-filter = { path = "../../diesel_filter/diesel-filter", features = ["serialize", "rocket"] }
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

The `filter` method can be used in conjunction with other diesel methods like `inner_join` and such.

```rust
Project::filter(&filters)
    .inner_join(clients::table)
    .select((projects::id, clients::name))
    .load::<ProjectResponse>(conn)
```

### With Rocket

With the `rocket` feature, the generated struct can be obtained from the request query parameters (dot notation `?filters.name=xxx`)

### With Actix

With the `actix` feature, the generated struct can be obtained from the request query parameters

N.B: unlike the `rocket` integration, the query parameters must be sent unscopped. e.g `?field=xxx&other=1`

### Pagination

The `diesel-pagination` crate exports a trait with `paginate` and `load_and_count` methods.

```rust
use diesel_pagination::Paginate;

Project::filter(&filters)
    .inner_join(clients::table)
    .select((projects::id, clients::name))
    .paginate(PaginationParams { page: Some(1), per_page: Some(10) })
    .load_and_count::<ProjectResponse>(conn)
```

`PaginationParams` can be used as an additional query parameters struct to the generated `[YourStruct]Filter` in `actix`/`axum`/`rocket`.

```rust
#[derive(Queryable, DieselFilter)]
#[diesel(table_name = projects)]
pub struct Project
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
