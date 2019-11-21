table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector};

    auth_tokens (id) {
        id -> Varchar,
        author -> Varchar,
        access_token -> Varchar,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector};

    modules (id) {
        id -> Int4,
        author -> Varchar,
        name -> Varchar,
        description -> Text,
        latest -> Nullable<Varchar>,
        search_vector -> Tsvector,
        featured -> Bool,
        source -> Nullable<Varchar>,
        redirect -> Nullable<Varchar>,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector};

    releases (id) {
        id -> Int4,
        module_id -> Int4,
        version -> Varchar,
        downloads -> Int4,
        code -> Text,
        published -> Timestamp,
    }
}

joinable!(releases -> modules (module_id));

allow_tables_to_appear_in_same_query!(
    auth_tokens,
    modules,
    releases,
);
