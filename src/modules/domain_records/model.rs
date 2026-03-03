use std::fs::OpenOptions;

use crate::modules::domain_records::{GenFromDomainrecord, Record};
use crate::modules::domains::model::Domain;
use crate::schema::domain_records;
use crate::utils::config;
use crate::utils::traits::Print;
use crate::{db::establish_connection, modules::domain_records::dtos::CreateDomainRecordDto};
use diesel::prelude::{Associations, Identifiable, Insertable, Queryable};
use diesel::result::Error;
use dotenv::dotenv;
use std::io::Write;
#[derive(Queryable, Identifiable, Associations)]
#[diesel(belongs_to(Domain))]
#[diesel(table_name = domain_records)]
pub struct DomainRecord {
    pub id: i32,
    pub host: String,
    pub ttl: Option<i64>,
    pub class: String,
    pub record_type: String,
    pub mx_preference: Option<i64>,
    pub value: String,
    pub soa_host_mail: Option<String>,
    pub soa_serial_number: Option<i64>,
    pub soa_refresh_time: Option<i64>,
    pub soa_retry_time: Option<i64>,
    pub soa_expire_time: Option<i64>,
    pub soa_negative_cache_ttl: Option<i64>,
    pub is_deleted: Option<bool>,
    pub domain_id: Option<i32>,
}

impl Print for DomainRecord {
    fn print(self: &Self) -> String {
        let mut domain_string_value = String::from("");
        dotenv().ok();
        let default_ttl = config().DEFAULT_TTL;
        domain_string_value.push_str(&format!(
            "\n\n$TTL {}\n$ORIGIN {}.\n\n",
            default_ttl, self.host
        ));

        domain_string_value
    }
}

#[derive(Insertable)]
#[diesel(belongs_to(Domain))]
#[diesel(table_name = domain_records)]
pub struct NewDomainRecord {
    pub ttl: Option<i64>,
    pub class: String,
    pub record_type: String,
    pub mx_preference: Option<i64>,
    pub value: String,
    pub domain_id: i32,
    pub is_deleted: Option<bool>,
    pub host: String,
    pub soa_host_mail: Option<String>,
    pub soa_serial_number: Option<i64>,
    pub soa_refresh_time: Option<i64>,
    pub soa_retry_time: Option<i64>,
    pub soa_expire_time: Option<i64>,
    pub soa_negative_cache_ttl: Option<i64>,
}

pub fn save_domain_record(
    data: CreateDomainRecordDto,
    record_domain_id: i32,
) -> Result<DomainRecord, Error> {
    use crate::schema::domain_records::dsl::*;
    use crate::schema::domains::dsl as domains_dsl;
    use diesel::prelude::*;
    let mut conn = establish_connection();

    let new_domain_record = NewDomainRecord {
        ttl: data.ttl,
        class: String::from("IN"),
        domain_id: record_domain_id,
        host: data.host,
        value: data.value,
        mx_preference: data.mx_preference,
        record_type: data.record_type,
        soa_expire_time: data.soa_expire_time,
        is_deleted: Some(false),
        soa_host_mail: data.soa_host_mail,
        soa_negative_cache_ttl: data.soa_negative_cache_ttl,
        soa_refresh_time: data.soa_refresh_time,
        soa_retry_time: data.soa_retry_time,
        soa_serial_number: data.soa_serial_number,
    };

    let owning_domain_name: String = domains_dsl::domains
        .find(record_domain_id)
        .select(domains_dsl::domain)
        .first::<String>(&mut conn)
        .expect("Failed to find domain for the given domain_id");

    println!("Domain id {}", new_domain_record.domain_id);

    let inserted_domain_record: DomainRecord = diesel::insert_into(domain_records)
        .values(&new_domain_record)
        .get_result(&mut conn)?;

    println!("Domain record id {}", inserted_domain_record.id);

    let domain_zonefile_path = format!(
        "{}/{}/db.{}",
        config().BIND9_PATH,
        owning_domain_name,
        owning_domain_name
    );

    let mut domain_zonefile = OpenOptions::new()
        .append(true)
        .open(&domain_zonefile_path)
        .expect("Error occurred while creating zone config");

    writeln!(
        domain_zonefile,
        "{}",
        Record::from_domain_record(&inserted_domain_record).print()
    )
    .expect("Error occurred while linking the new domain record to named config.");

    Ok(inserted_domain_record)
}
