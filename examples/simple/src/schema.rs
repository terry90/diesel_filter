table! {
    thingies (id) {
        id -> Uuid,
        name -> Varchar,
        num32 -> Int4,
        option_num32 -> Nullable<Int4>,
        num64 -> Int8,
        option_num64 -> Nullable<Int8>,
        text -> Varchar,
        option_text -> Nullable<Varchar>,
        custom -> Varchar,
        option_custom -> Nullable<Varchar>,
        multiple_custom -> Varchar,
    }
}
