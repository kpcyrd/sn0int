table! {
    accounts (id) {
        id -> Integer,
        value -> Text,
        service -> Text,
        username -> Text,
        displayname -> Nullable<Text>,
        email -> Nullable<Text>,
        url -> Nullable<Text>,
        last_seen -> Nullable<Timestamp>,
        unscoped -> Bool,
        phonenumber -> Nullable<Text>,
        profile_pic -> Nullable<Text>,
        birthday -> Nullable<Text>,
    }
}

table! {
    activity (id) {
        id -> Integer,
        topic -> Text,
        time -> Timestamp,
        uniq -> Nullable<Text>,
        latitude -> Nullable<Float>,
        longitude -> Nullable<Float>,
        radius -> Nullable<Integer>,
        content -> Text,
    }
}

table! {
    autonoscope (id) {
        id -> Integer,
        object -> Text,
        value -> Text,
        scoped -> Bool,
    }
}

table! {
    breach_emails (id) {
        id -> Integer,
        breach_id -> Integer,
        email_id -> Integer,
        password -> Nullable<Text>,
    }
}

table! {
    breaches (id) {
        id -> Integer,
        value -> Text,
        unscoped -> Bool,
    }
}

table! {
    cryptoaddrs (id) {
        id -> Integer,
        value -> Text,
        currency -> Nullable<Text>,
        denominator -> Nullable<Integer>,
        balance -> Nullable<BigInt>,
        received -> Nullable<BigInt>,
        first_seen -> Nullable<Timestamp>,
        last_withdrawal -> Nullable<Timestamp>,
        unscoped -> Bool,
        description -> Nullable<Text>,
    }
}

table! {
    devices (id) {
        id -> Integer,
        value -> Text,
        name -> Nullable<Text>,
        hostname -> Nullable<Text>,
        vendor -> Nullable<Text>,
        unscoped -> Bool,
        last_seen -> Nullable<Timestamp>,
    }
}

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
        displayname -> Nullable<Text>,
    }
}

table! {
    images (id) {
        id -> Integer,
        value -> Text,
        filename -> Nullable<Text>,
        mime -> Nullable<Text>,
        width -> Nullable<Integer>,
        height -> Nullable<Integer>,
        created -> Nullable<Timestamp>,
        latitude -> Nullable<Float>,
        longitude -> Nullable<Float>,
        nudity -> Nullable<Float>,
        ahash -> Nullable<Text>,
        dhash -> Nullable<Text>,
        phash -> Nullable<Text>,
        unscoped -> Bool,
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
        description -> Nullable<Text>,
        reverse_dns -> Nullable<Text>,
    }
}

table! {
    netblocks (id) {
        id -> Integer,
        family -> Text,
        value -> Text,
        unscoped -> Bool,
        asn -> Nullable<Integer>,
        as_org -> Nullable<Text>,
        description -> Nullable<Text>,
    }
}

table! {
    network_devices (id) {
        id -> Integer,
        network_id -> Integer,
        device_id -> Integer,
        ipaddr -> Nullable<Text>,
        last_seen -> Nullable<Timestamp>,
    }
}

table! {
    networks (id) {
        id -> Integer,
        value -> Text,
        unscoped -> Bool,
        latitude -> Nullable<Float>,
        longitude -> Nullable<Float>,
        description -> Nullable<Text>,
    }
}

table! {
    phonenumbers (id) {
        id -> Integer,
        value -> Text,
        name -> Nullable<Text>,
        unscoped -> Bool,
        valid -> Nullable<Bool>,
        last_online -> Nullable<Timestamp>,
        country -> Nullable<Text>,
        carrier -> Nullable<Text>,
        line -> Nullable<Text>,
        is_ported -> Nullable<Bool>,
        last_ported -> Nullable<Timestamp>,
        caller_name -> Nullable<Text>,
        caller_type -> Nullable<Text>,
    }
}

table! {
    ports (id) {
        id -> Integer,
        ip_addr_id -> Integer,
        value -> Text,
        ip_addr -> Text,
        port -> Integer,
        protocol -> Text,
        status -> Nullable<Text>,
        unscoped -> Bool,
        banner -> Nullable<Text>,
        service -> Nullable<Text>,
        version -> Nullable<Text>,
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
    ttls (id) {
        id -> Integer,
        family -> Text,
        key -> Integer,
        value -> Text,
        expire -> Timestamp,
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

joinable!(breach_emails -> breaches (breach_id));
joinable!(breach_emails -> emails (email_id));
joinable!(network_devices -> devices (device_id));
joinable!(network_devices -> networks (network_id));
joinable!(ports -> ipaddrs (ip_addr_id));
joinable!(subdomain_ipaddrs -> ipaddrs (ip_addr_id));
joinable!(subdomain_ipaddrs -> subdomains (subdomain_id));
joinable!(subdomains -> domains (domain_id));
joinable!(urls -> subdomains (subdomain_id));

allow_tables_to_appear_in_same_query!(
    accounts,
    activity,
    autonoscope,
    breach_emails,
    breaches,
    cryptoaddrs,
    devices,
    domains,
    emails,
    images,
    ipaddrs,
    netblocks,
    network_devices,
    networks,
    phonenumbers,
    ports,
    subdomain_ipaddrs,
    subdomains,
    ttls,
    urls,
);
