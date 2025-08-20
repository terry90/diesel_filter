use darling::{FromDeriveInput, FromField, FromMeta, ast, util::Ignored};
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{ToTokens, quote};
use syn::{Attribute, DeriveInput, Meta, Type, parse_macro_input};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(diesel))]
struct DieselAttrs {
    table_name: Ident,
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(filter), supports(struct_named))]
struct DieselFilterAttrs {
    ident: Ident,
    data: ast::Data<Ignored, DieselFilterField>,
}

#[derive(Debug, FromField)]
#[darling(forward_attrs(filter))]
struct DieselFilterField {
    ident: Option<Ident>,
    ty: Type,
    attrs: Vec<Attribute>,
}

#[derive(Debug, FromMeta, Default)]
struct DieselFilterFieldAttrs {
    #[darling(default)]
    multiple: bool,
    #[darling(default)]
    insensitive: bool,
    #[darling(default)]
    substring: bool,
}

struct DieselFilterFieldAttrsOuter(DieselFilterFieldAttrs);

impl FromMeta for DieselFilterFieldAttrsOuter {
    fn from_word() -> darling::Result<Self> {
        Ok(Self(Default::default()))
    }

    fn from_meta(item: &Meta) -> darling::Result<Self> {
        match item {
            Meta::Path(_) => Self::from_word(),
            _ => DieselFilterFieldAttrs::from_meta(item).map(Self),
        }
    }
}

// https://stackoverflow.com/a/77040924/746914
fn option_type(ty: &Type) -> Option<&Type> {
    let Type::Path(ty) = ty else { return None };
    if ty.qself.is_some() {
        return None;
    }

    let ty = &ty.path;

    if ty.segments.is_empty() || ty.segments.last().unwrap().ident != "Option" {
        return None;
    }

    if !(ty.segments.len() == 1
        || (ty.segments.len() == 3
            && ["core", "std"].contains(&ty.segments[0].ident.to_string().as_str())
            && ty.segments[1].ident == "option"))
    {
        return None;
    }

    let last_segment = ty.segments.last().unwrap();
    let syn::PathArguments::AngleBracketed(generics) = &last_segment.arguments else {
        return None;
    };
    if generics.args.len() != 1 {
        return None;
    }
    let syn::GenericArgument::Type(inner_type) = &generics.args[0] else {
        return None;
    };

    Some(inner_type)
}

#[proc_macro_derive(DieselFilter, attributes(filter, table_name))]
pub fn diesel_filter_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let DieselAttrs { table_name } = match DieselAttrs::from_derive_input(&input) {
        Ok(ok) => ok,
        Err(err) => return err.write_errors().into(),
    };

    let DieselFilterAttrs {
        ident: struct_ident,
        data,
    } = match DieselFilterAttrs::from_derive_input(&input) {
        Ok(ok) => ok,
        Err(err) => return err.write_errors().into(),
    };

    let filter_struct_ident = Ident::new(&format!("{struct_ident}Filters"), struct_ident.span());

    let mut errors = vec![];
    let mut fields = vec![];
    let mut queries = vec![];
    let mut uses = vec![];
    let mut has_multiple = false;

    for field_filter in data.take_struct().expect("Expected struct data").fields {
        let Some(attr) = field_filter.attrs.into_iter().next() else {
            continue;
        };
        let filter_attr = match DieselFilterFieldAttrsOuter::from_meta(&attr.meta) {
            Ok(ok) => ok.0,
            Err(err) => {
                errors.push(err.write_errors());
                continue;
            }
        };

        let field = field_filter
            .ident
            .expect("Expected all fields to have identifiers");

        let ty = option_type(&field_filter.ty)
            .unwrap_or(&field_filter.ty)
            .to_owned();
        let ty: Ident = Ident::new(
            ty.to_token_stream().to_string().replace(' ', "").as_str(),
            Span::call_site(),
        );

        let q = if filter_attr.multiple {
            has_multiple = true;

            #[allow(unused_mut)]
            let mut field_attributes: Vec<proc_macro2::TokenStream> = vec![];

            #[cfg(feature = "utoipa")]
            field_attributes.push(quote! { #[param(value_type = String)] });

            #[cfg(feature = "rocket")]
            field_attributes.push(quote! { #[field(default = Option::None)] });

            #[cfg(any(feature = "actix", feature = "axum"))]
            {
                let serde_as_path = format!(
                    "Option<::diesel_filter::serde_with::StringWithSeparator::<::diesel_filter::serde_with::formats::CommaSeparator, {}>>",
                    ty
                );
                field_attributes.push(quote! { #[serde_as(as = #serde_as_path)] });
            }

            fields.push(quote! {
                #( #field_attributes )*
                pub #field: Option<Vec<#ty>>,
            });

            match (filter_attr.insensitive, filter_attr.substring) {
                (false, false) => {
                    quote! { #table_name::#field.eq(any(filter)) }
                }
                (false, true) => {
                    quote! {
                        #table_name::#field.like(any(
                            filter.iter().map(|f| format!("%{}%", f)).collect::<Vec<_>>()
                        ))
                    }
                }
                (true, false) => {
                    quote! { #table_name::#field.ilike(any(filter)) }
                }
                (true, true) => {
                    quote! {
                        #table_name::#field.ilike(any(
                            filter.iter().map(|f| format!("%{}%", f)).collect::<Vec<_>>()
                        ))
                    }
                }
            }
        } else {
            fields.push(quote! {
                pub #field: Option<#ty>,
            });
            match (filter_attr.insensitive, filter_attr.substring) {
                (false, false) => {
                    quote! { #table_name::#field.eq(filter) }
                }
                (false, true) => {
                    quote! { #table_name::#field.like(format!("%{}%", filter)) }
                }
                (true, false) => {
                    quote! { #table_name::#field.ilike(filter) }
                }
                (true, true) => {
                    quote! { #table_name::#field.ilike(format!("%{}%", filter)) }
                }
            }
        };

        queries.push(quote! {
            if let Some(filter) = filters.#field {
                query = query.filter(#q);
            }
        });
    }

    if has_multiple {
        uses.push(quote! { use diesel::dsl::any; })
    }

    let mut extra_derive = vec![];
    extra_derive.push(quote!(Debug));
    extra_derive.push(quote!(Default));

    #[cfg(feature = "utoipa")]
    extra_derive.push(quote!(utoipa::IntoParams));

    #[cfg(feature = "rocket")]
    let filters_struct = quote! {
        #[derive(FromForm, #( #extra_derive, )*)]
        pub struct #filter_struct_ident {
            #( #fields )*
        }
    };

    #[cfg(any(feature = "actix", feature = "axum"))]
    let filters_struct = quote! {
        #[::diesel_filter::serde_with::serde_as(crate = "::diesel_filter::serde_with")]
        #[derive(serde::Deserialize, #( #extra_derive, )*)]
        pub struct #filter_struct_ident {
            #( #fields )*
        }
    };

    #[cfg(not(any(feature = "rocket", feature = "actix", feature = "axum")))]
    let filters_struct = quote! {
        #[derive(#( #extra_derive, )*)]
        pub struct #filter_struct_ident {
            #( #fields )*
        }
    };

    let filter_func = quote! {
        pub fn filter<'a>(filters: #filter_struct_ident) -> #table_name::BoxedQuery<'a, diesel::pg::Pg> {
            #( #uses )*
            let mut query = #table_name::table.into_boxed();

            #( #queries )*

            query
        }
    };

    if errors.is_empty() {
        quote! {
            #filters_struct

            impl #struct_ident {
                #filter_func
            }
        }
    } else {
        quote! {
            #( #errors )*
        }
    }
    .into()
}
