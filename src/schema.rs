table! {
    domains (id) {
        id -> Nullable<Integer>,
        value -> Text,
    }
}

table! {
    ipaddrs (id) {
        id -> Nullable<Integer>,
        family -> Text,
        value -> Text,
    }
}

table! {
    subdomain_ipaddrs (id) {
        id -> Nullable<Integer>,
        subdomain_id -> Integer,
        ip_addr_id -> Integer,
    }
}

table! {
    subdomains (id) {
        id -> Nullable<Integer>,
        domain_id -> Integer,
        value -> Text,
    }
}

table! {
    urls (id) {
        id -> Nullable<Integer>,
        subdomain_id -> Integer,
        value -> Text,
        status -> Nullable<Integer>,
        body -> Nullable<Binary>,
    }
}

joinable!(subdomain_ipaddrs -> domains (subdomain_id));
joinable!(subdomain_ipaddrs -> ipaddrs (ip_addr_id));
joinable!(subdomains -> domains (domain_id));
joinable!(urls -> domains (subdomain_id));

allow_tables_to_appear_in_same_query!(
    domains,
    ipaddrs,
    subdomain_ipaddrs,
    subdomains,
    urls,
);
