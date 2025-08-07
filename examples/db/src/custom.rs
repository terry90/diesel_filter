use diesel_derive_newtype::DieselNewType;
use serde::{Deserialize, Serialize};
use std::{convert::Infallible, str::FromStr};

#[derive(Debug, Serialize, Deserialize, DieselNewType)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct CustomType(String);

impl FromStr for CustomType {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_owned()))
    }
}
