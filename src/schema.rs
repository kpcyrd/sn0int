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
    }
}

table! {
    ipaddrs (id) {
        id -> Integer,
        family -> Text,
        value -> Text,
        unscoped -> Bool,
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
    }
}

table! {
    urls (id) {
        id -> Integer,
        subdomain_id -> Integer,
        value -> Text,
        status -> Nullable<Integer>,
        body -> Nullable<Binary>,
        unscoped -> Bool,
    }
}

joinable!(subdomain_ipaddrs -> domains (subdomain_id));
joinable!(subdomain_ipaddrs -> ipaddrs (ip_addr_id));
joinable!(subdomains -> domains (domain_id));
joinable!(urls -> domains (subdomain_id));

allow_tables_to_appear_in_same_query!(
    domains,
    emails,
    ipaddrs,
    subdomain_ipaddrs,
    subdomains,
    urls,
);
