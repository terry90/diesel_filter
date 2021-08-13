use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, Data, DeriveInput, Fields, Lit, Meta, NestedMeta, Path, Type, TypePath,
};

struct Filter {
    pub name: Ident,
    pub ty: FilterableType,
    pub kind: FilterKind,
}

enum FilterableType {
    String,
    Uuid,
    Unknown(String),
}

enum FilterKind {
    Basic,
    Substr,
    Insensitive,
    SubstrInsensitive,
}

impl From<Vec<NestedMeta>> for FilterKind {
    fn from(m: Vec<NestedMeta>) -> Self {
        let meta = m
            .into_iter()
            .filter_map(|m| match m {
                NestedMeta::Meta(m) => Some(m.path().to_owned()),
                _ => None,
            })
            .collect::<Vec<_>>();

        let matches = |m: &Vec<Path>, tested: &[&str]| {
            m.iter().all(|m| tested.iter().any(|t| m.is_ident(t)))
                && tested.iter().all(|t| m.iter().any(|m| m.is_ident(t)))
        };

        if matches(&meta, &["substring", "insensitive"]) {
            return Self::SubstrInsensitive;
        } else if matches(&meta, &["substring"]) {
            return Self::Substr;
        } else if matches(&meta, &["insensitive"]) {
            return Self::Insensitive;
        }

        Self::Basic
    }
}

impl From<&TypePath> for FilterableType {
    fn from(ty: &TypePath) -> Self {
        match ty.to_token_stream().to_string().replace(" ", "").as_str() {
            "String" => Self::String,
            "Uuid" => Self::Uuid,
            "uuid::Uuid" => Self::Uuid,
            "Option<String>" => Self::String,
            "Option<Uuid>" => Self::Uuid,
            "Option<uuid::Uuid>" => Self::Uuid,
            other => Self::Unknown(other.to_string()),
        }
    }
}

impl Into<Ident> for FilterableType {
    fn into(self) -> Ident {
        match self {
            FilterableType::String => Ident::new("String", Span::call_site()),
            FilterableType::Uuid => Ident::new("Uuid", Span::call_site()),
            FilterableType::Unknown(ty) => panic!("the type {} is not supported", ty),
        }
    }
}

#[proc_macro_derive(DieselFilter, attributes(filter, table_name, pagination))]
pub fn filter(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let table_name_from_attribute = &input
        .attrs
        .iter()
        .map(|m| m.parse_meta().unwrap())
        .filter(|m| m.path().is_ident("table_name"))
        .last();
    let pagination = input
        .attrs
        .iter()
        .map(|m| m.parse_meta().unwrap())
        .filter(|m| m.path().is_ident("pagination"))
        .last()
        .is_some();
    match table_name_from_attribute {
        Some(table_name) => {
            let table_name = match table_name {
                Meta::NameValue(name_value) => match name_value.lit.clone() {
                    Lit::Str(value) => value.parse::<syn::Path>().unwrap(),
                    _ => panic!("the attribute should be in the form of table_name = \"value\""),
                },
                _ => panic!("the attribute should be in the form of table_name = \"value\""),
            };

            let struct_name = input.ident;
            let mut filters = vec![];

            if let Data::Struct(data) = input.data {
                if let Fields::Named(fields) = data.fields {
                    for field in fields.named {
                        match field.ident {
                            Some(name) => {
                                let field_type = field.ty;
                                for attr in field.attrs.into_iter() {
                                    if !attr.path.is_ident("filter") {
                                        continue;
                                    }
                                    let kind = match attr.parse_meta().unwrap() {
                                        Meta::List(te) => FilterKind::from(
                                            te.nested.into_iter().collect::<Vec<_>>(),
                                        ),
                                        Meta::Path(_) => FilterKind::Basic,
                                        _ => continue,
                                    };

                                    if let Type::Path(ty) = &field_type {
                                        let ty = FilterableType::from(ty);
                                        let name = name.clone();

                                        filters.push(Filter { name, ty, kind });
                                        continue;
                                    }
                                    panic!("this type is not supported");
                                }
                            }
                            None => continue,
                        }
                    }
                }
            }

            let filter_struct_ident = Ident::new(
                &format!("{}Filters", struct_name.to_string()),
                struct_name.span(),
            );

            if filters.len() == 0 {
                panic!(
                    "please annotate at least one field to filter with #[filter] on your struct"
                );
            }

            let mut fields = vec![];
            let mut queries = vec![];
            for filter in filters {
                let field = filter.name;
                let ty: Ident = filter.ty.into();
                let kind = filter.kind;

                fields.push(quote! {
                    pub #field: Option<#ty>,
                });

                match kind {
                    FilterKind::Basic => {
                        queries.push(quote! {
                            if let Some(ref filter) = filters.#field {
                                query = query.filter(#table_name::#field.eq(filter));
                            }
                        });
                    }
                    FilterKind::Substr => {
                        queries.push(quote! {
                            if let Some(ref filter) = filters.#field {
                                query = query.filter(#table_name::#field.like(format!("%{}%", filter)));
                            }
                        });
                    }
                    FilterKind::Insensitive => {
                        queries.push(quote! {
                            if let Some(ref filter) = filters.#field {
                                query = query.filter(#table_name::#field.ilike(filter));
                            }
                        });
                    }
                    FilterKind::SubstrInsensitive => {
                        queries.push(quote! {
                            if let Some(ref filter) = filters.#field {
                                query = query.filter(#table_name::#field.ilike(format!("%{}%", filter)));
                            }
                        });
                    }
                }
            }

            if pagination {
                fields.push(quote! {
                    pub page: Option<i64>,
                    pub per_page: Option<i64>,
                });
            }

            #[cfg(feature = "rocket")]
            let filters_struct = quote! {
                #[derive(FromForm)]
                pub struct #filter_struct_ident {
                    #( #fields )*
                }
            };

            #[cfg(not(feature = "rocket"))]
            let filters_struct = quote! {
                pub struct #filter_struct_ident {
                    #( #fields )*
                }
            };

            let expanded = match pagination {
                true => {
                    quote! {
                        #filters_struct

                        impl #struct_name {
                            pub fn filtered(filters: &#filter_struct_ident, conn: &PgConnection) -> Result<(Vec<#struct_name>, i64), diesel::result::Error> {
                                Self::filter(filters)
                                  .paginate(filters.page)
                                  .per_page(filters.per_page)
                                  .load_and_count::<#struct_name>(conn)
                            }

                            pub fn filter<'a>(filters: &'a #filter_struct_ident) -> #table_name::BoxedQuery<'a, diesel::pg::Pg> {
                                use crate::schema::#table_name;

                                let mut query = #table_name::table.into_boxed();

                                #( #queries )*

                                query
                            }
                        }
                    }
                }
                false => {
                    quote! {
                        #[derive(FromForm)]
                        pub struct #filter_struct_ident {
                            #( #fields )*
                        }

                        impl #struct_name {
                            pub fn filtered(filters: &#filter_struct_ident, conn: &PgConnection) -> Result<Vec<#struct_name>, diesel::result::Error> {
                                Self::filter(filters).load::<#struct_name>(conn)
                            }

                            pub fn filter<'a>(filters: &'a #filter_struct_ident) -> #table_name::BoxedQuery<'a, diesel::pg::Pg> {
                                use crate::schema::#table_name;

                                let mut query = #table_name::table.into_boxed();

                                #( #queries )*

                                query
                            }
                        }
                    }
                }
            };

            TokenStream::from(expanded)
        }
        None => panic!("please provide table_name attribute"),
    }
}
