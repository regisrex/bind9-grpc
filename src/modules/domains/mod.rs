use dotenv::dotenv;
use std::{env, io::Error};

use crate::{
    modules::{
        self,
        domain_records::{Record, SOAProperties},
    },
    utils::{
        config,
        traits::{Defaults, Initialize, Print},
    },
};

pub mod model;
pub mod service;

pub struct DomainConfig {
    pub domain: String,
    pub records: Vec<Record>,
}

impl Initialize for DomainConfig {
    // provide  ip address of the server to host the domain to, but it's now done on the client's level
    fn new(domain_name: &String, _ip_address: &String) -> Result<DomainConfig, Error> {
        dotenv().ok();
        let default_nameserver: String = config().DNS_HOSTNAME;

        let soa_record = Record {
            id: String::from("1"),
            name: String::from("@"),
            domain_id: None,
            ttl: Some(config().DEFAULT_TTL as u64),
            class: String::from("IN"),
            record_type: modules::domain_records::DNSRecordType::SOA,
            mx_preference: None,
            hostname: default_nameserver,
            soa_properties: Some(SOAProperties::defaults(domain_name.to_string())),
            is_deleted: None,
        };
        let domain = DomainConfig {
            domain: domain_name.to_string(),
            records: vec![soa_record.clone()],
        };
        Ok(domain)
    }
}
impl Print for DomainConfig {
    fn print(self: &Self) -> String {
        let mut domain_string_value = String::from("");
        dotenv().ok();
        let default_ttl =
            env::var("DEFAULT_TTL").expect("Invalid .env config, missing : DEFAULT_TTL");

        domain_string_value.push_str(&format!(
            "\n\n$TTL {}\n$ORIGIN {}.\n\n",
            default_ttl, self.domain
        ));
        for domain_record in self.records.clone() {
            domain_string_value.push_str(&domain_record.print());
        }

        domain_string_value
    }
}
