table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector};

    auth_tokens (token) {
        token -> Varchar,
        author -> Varchar,
        created_at -> Timestamp,
        used_at -> Timestamp,
        oauth -> Varchar,
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
        search_tokens -> Nullable<Tsvector>,
        latest -> Int4,
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
    }
}

allow_tables_to_appear_in_same_query!(
    auth_tokens,
    modules,
    releases,
);
