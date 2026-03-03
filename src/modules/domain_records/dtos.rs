use diesel::prelude::{Identifiable, Insertable, Queryable};

use crate::{modules::domain_records::Record, schema::domain_records};

#[derive(Debug)]
pub struct CreateDomainRecordDto {
    pub ttl: Option<i64>,
    pub domain_id: Option<i32>,
    pub class: String,
    pub host: String,
    pub record_type: String,
    pub mx_preference: Option<i64>,
    pub value: String,
    pub soa_host_mail: Option<String>,
    pub soa_serial_number: Option<i64>,
    pub soa_refresh_time: Option<i64>,
    pub soa_retry_time: Option<i64>,
    pub soa_expire_time: Option<i64>,
    pub soa_negative_cache_ttl: Option<i64>,
}

pub trait InitializeFromRecord {
    fn from_record(data: Record) -> Self;
}

impl InitializeFromRecord for CreateDomainRecordDto {
    fn from_record(data: Record) -> Self {
        CreateDomainRecordDto {
            ttl: data.ttl.map(|t| t as i64),
            domain_id: data.domain_id,
            class: data.class,
            record_type: data.record_type.to_string(),
            mx_preference: data.mx_preference.map(|m| m as i64),
            value: data.hostname,
            host: data.name,
            soa_host_mail: data
                .soa_properties
                .as_ref()
                .map(|soa| soa.host_mail.clone()),
            soa_serial_number: data
                .soa_properties
                .as_ref()
                .and_then(|soa| soa.serial_number.parse::<i64>().ok()),
            soa_refresh_time: data
                .soa_properties
                .as_ref()
                .map(|soa| soa.refresh_time as i64),
            soa_retry_time: data
                .soa_properties
                .as_ref()
                .map(|soa| soa.retry_time as i64),
            soa_expire_time: data
                .soa_properties
                .as_ref()
                .map(|soa| soa.expire_time as i64),
            soa_negative_cache_ttl: data.soa_properties.map(|soa| soa.negative_cache_ttl as i64),
        }
    }
}
