// @generated automatically by Diesel CLI.

diesel::table! {
    domain_records (id) {
        id -> Int4,
        host -> Text,
        ttl -> Nullable<Int8>,
        class -> Text,
        record_type -> Text,
        mx_preference -> Nullable<Int8>,
        value -> Text,
        soa_host_mail -> Nullable<Text>,
        soa_serial_number -> Nullable<Int8>,
        soa_refresh_time -> Nullable<Int8>,
        soa_retry_time -> Nullable<Int8>,
        soa_expire_time -> Nullable<Int8>,
        soa_negative_cache_ttl -> Nullable<Int8>,
        is_deleted -> Nullable<Bool>,
        domain_id -> Nullable<Int4>,
    }
}

diesel::table! {
    domains (id) {
        id -> Int4,
        domain -> Text,
        is_deleted -> Nullable<Bool>,
    }
}

diesel::joinable!(domain_records -> domains (domain_id));

diesel::allow_tables_to_appear_in_same_query!(
    domain_records,
    domains,
);
