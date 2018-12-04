table! {
    domains (id) {
        id -> Integer,
        value -> Text,
        unscoped -> Bool,
    }
}

table! {
    emails (id) {
        id -> Integer,
        value -> Text,
        unscoped -> Bool,
        valid -> Nullable<Bool>,
    }
}

table! {
    ipaddrs (id) {
        id -> Integer,
        family -> Text,
        value -> Text,
        unscoped -> Bool,
        continent -> Nullable<Text>,
        continent_code -> Nullable<Text>,
        country -> Nullable<Text>,
        country_code -> Nullable<Text>,
        city -> Nullable<Text>,
        latitude -> Nullable<Float>,
        longitude -> Nullable<Float>,
        asn -> Nullable<Integer>,
        as_org -> Nullable<Text>,
    }
}

table! {
    subdomain_ipaddrs (id) {
        id -> Integer,
        subdomain_id -> Integer,
        ip_addr_id -> Integer,
    }
}

table! {
    subdomains (id) {
        id -> Integer,
        domain_id -> Integer,
        value -> Text,
        unscoped -> Bool,
        resolvable -> Nullable<Bool>,
    }
}

table! {
    urls (id) {
        id -> Integer,
        subdomain_id -> Integer,
        value -> Text,
        path -> Text,
        status -> Nullable<Integer>,
        body -> Nullable<Binary>,
        unscoped -> Bool,
        online -> Nullable<Bool>,
        title -> Nullable<Text>,
        redirect -> Nullable<Text>,
    }
}

joinable!(subdomain_ipaddrs -> ipaddrs (ip_addr_id));
joinable!(subdomain_ipaddrs -> subdomains (subdomain_id));
joinable!(subdomains -> domains (domain_id));
joinable!(urls -> subdomains (subdomain_id));

allow_tables_to_appear_in_same_query!(
    domains,
    emails,
    ipaddrs,
    subdomain_ipaddrs,
    subdomains,
    urls,
);
