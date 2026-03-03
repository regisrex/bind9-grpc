pub mod dtos;
pub mod model;
pub mod service;
use dotenv::dotenv;
use std::{env, fmt};

use crate::{
    modules::domain_records::model::DomainRecord,
    utils::{
        config, generate_incremental_zone_serial,
        traits::{Defaults, Print},
    },
};

#[derive(Debug, Copy, Clone)]
pub enum DNSRecordType {
    A,     // IPv4 address
    NS,    // Name server
    CNAME, // Canonical name
    SOA,   // Start of authority
    PTR,   // Pointer record
    MX,    // Mail exchange
    TXT,   // Text record
    AAAA,  // IPv6 address
    SRV,   // Service record
}

#[derive(Debug, Clone)]
pub struct SOAProperties {
    pub host_mail: String,
    pub serial_number: String,
    pub refresh_time: u64,
    pub retry_time: u64,
    pub expire_time: u64,
    pub negative_cache_ttl: u64,
}

impl Defaults for SOAProperties {
    fn defaults(domain_name: String) -> Self
    where
        Self: Sized,
    {
        dotenv().ok();
        let host_email_address: String = config().HOSTMASTER_EMAIL;
        let serial_number = generate_incremental_zone_serial(&domain_name)
            .expect("Serial number generation failed");

        SOAProperties {
            host_mail: host_email_address,
            serial_number: serial_number,
            refresh_time: 30,
            retry_time: 10,
            expire_time: 30,
            negative_cache_ttl: 30,
        }
    }
}

#[derive(Clone)]
pub struct Record {
    pub id: String,
    pub name: String,
    pub domain_id: Option<i32>,
    pub ttl: Option<u64>, // default is zone's ttl
    pub class: String,
    pub record_type: DNSRecordType, // Enum for available DNS record types
    pub mx_preference: Option<u64>,
    pub hostname: String, // domain ending with a dot or an ip address.
    pub soa_properties: Option<SOAProperties>,
    pub is_deleted: Option<bool>,
}

impl fmt::Display for DNSRecordType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            DNSRecordType::A => "A",
            DNSRecordType::NS => "NS",
            DNSRecordType::CNAME => "CNAME",
            DNSRecordType::SOA => "SOA",
            DNSRecordType::PTR => "PTR",
            DNSRecordType::MX => "MX",
            DNSRecordType::TXT => "TXT",
            DNSRecordType::AAAA => "AAAA",
            DNSRecordType::SRV => "SRV",
        };
        write!(f, "{}", s)
    }
}

impl Print for Record {
    fn print(self: &Self) -> String {
        let mut record_string_value = String::new();

        // Handle SOA records with proper formatting
        if let DNSRecordType::SOA = self.record_type {
            if let Some(properties) = &self.soa_properties {
                // SOA records have a specific format: hostname TTL class SOA mname rname (serial refresh retry expire minimum)
                // Format SOA record according to RFC standards
                let mname = if self.name.ends_with('.') {
                    self.name.clone()
                } else if self.name.trim() == "@" {
                    format!("{}.", self.hostname)
                } else {
                    format!("{}.", self.name)
                };

                let rname = if properties.host_mail.contains('@') {
                    // Convert email format to DNS format: user@domain.com -> user.domain.com.
                    format!("{}.", properties.host_mail.replace('@', "."))
                } else if properties.host_mail.ends_with('.') {
                    properties.host_mail.clone()
                } else {
                    format!("{}.", properties.host_mail)
                };

                record_string_value = format!(
                    "{}\t{}\t{}\tSOA\t{}\t{}\t(\n\t\t{}\n\t\t{}\n\t\t{}\n\t\t{}\n\t\t{}\n\t)",
                    self.name,
                    self.ttl
                        .map(|t| t.to_string())
                        .unwrap_or_else(|| "".to_string()),
                    self.class,
                    mname, // Primary nameserver with trailing dot
                    rname, // Responsible person in DNS format with trailing dot
                    properties.serial_number,
                    properties.refresh_time,
                    properties.retry_time,
                    properties.expire_time,
                    properties.negative_cache_ttl
                );
            }
        } else {
            // For non-SOA records, use the standard format
            let mx_preference = if let DNSRecordType::MX = self.record_type {
                match self.mx_preference {
                    Some(v) => v.to_string(),
                    None => String::from("60"),
                }
            } else {
                String::new()
            };
            let hostname = if let DNSRecordType::TXT = self.record_type {
                format!("\"{}\"", self.hostname)
            } else if matches!(self.record_type, DNSRecordType::A | DNSRecordType::AAAA)
                && self.hostname.parse::<std::net::IpAddr>().is_ok()
            {
                self.hostname.clone()
            } else if matches!(
                self.record_type,
                DNSRecordType::A
                    | DNSRecordType::AAAA
                    | DNSRecordType::CNAME
                    | DNSRecordType::MX
                    | DNSRecordType::NS
                    | DNSRecordType::SOA
                    | DNSRecordType::PTR
            ) {
                if self.hostname.ends_with('.') {
                    self.hostname.clone()
                } else {
                    format!("{}.", self.hostname)
                }
            } else {
                self.hostname.clone()
            };

            record_string_value = format!(
                "{}\t{}\t{}\t{}\t{}\t{}",
                self.name,
                self.ttl
                    .map(|t| t.to_string())
                    .unwrap_or_else(|| "".to_string()),
                self.class,
                self.record_type.to_string(),
                mx_preference,
                hostname
            );
        }

        // Add comment with record ID
        record_string_value.push_str(&format!("\t;{}", self.id));

        // Add newline
        record_string_value.push('\n');

        record_string_value
    }
}

// Add this trait definition
pub trait GenFromDomainrecord {
    fn from_domain_record(data: &DomainRecord) -> Self;
}

impl GenFromDomainrecord for Record {
    fn from_domain_record(data: &DomainRecord) -> Record {
        // Convert DNSRecordType from string to enum
        let record_type = match data.record_type.as_str() {
            "A" => DNSRecordType::A,
            "NS" => DNSRecordType::NS,
            "CNAME" => DNSRecordType::CNAME,
            "SOA" => DNSRecordType::SOA,
            "PTR" => DNSRecordType::PTR,
            "MX" => DNSRecordType::MX,
            "TXT" => DNSRecordType::TXT,
            "AAAA" => DNSRecordType::AAAA,
            "SRV" => DNSRecordType::SRV,
            _ => panic!("Invalid record type: {}", data.record_type),
        };

        // Convert SOA properties if they exist
        let soa_properties = if data.soa_host_mail.is_some() {
            Some(SOAProperties {
                host_mail: data.soa_host_mail.clone().unwrap(),
                serial_number: data.soa_serial_number.unwrap_or(0).to_string(),
                refresh_time: data.soa_refresh_time.unwrap_or(0) as u64,
                retry_time: data.soa_retry_time.unwrap_or(0) as u64,
                expire_time: data.soa_expire_time.unwrap_or(0) as u64,
                negative_cache_ttl: data.soa_negative_cache_ttl.unwrap_or(0) as u64,
            })
        } else {
            None
        };

        Record {
            id: data.id.to_string(),
            name: data.host.clone(),
            // Fix: domain_id in Record is i32, but data.domain_id is Option<i32>
            // Use unwrap_or(0) to provide a default value if None
            domain_id: data.domain_id,
            ttl: data.ttl.map(|t| t as u64), // Convert i64 to u64
            class: data.class.clone(),
            is_deleted: data.is_deleted,
            record_type,
            mx_preference: data.mx_preference.map(|m| m as u64), // Convert i64 to u64
            hostname: data.value.clone(),
            soa_properties,
        }
    }
}

// Add reverse conversion trait
pub trait GenFromRecord {
    fn from_record(data: Record) -> Self;
}

impl GenFromRecord for DomainRecord {
    fn from_record(data: Record) -> DomainRecord {
        DomainRecord {
            id: data.id.parse().unwrap_or(0),
            host: data.name,
            ttl: data.ttl.map(|t| t as i64), // Convert u64 to i64
            class: data.class,
            record_type: data.record_type.to_string(),
            mx_preference: data.mx_preference.map(|m| m as i64), // Convert u64 to i64
            value: data.hostname,
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
            soa_negative_cache_ttl: data
                .soa_properties
                .as_ref()
                .map(|soa| soa.negative_cache_ttl as i64),
            is_deleted: data.is_deleted,
            domain_id: None, // This needs to be set separately as it's not part of Record
        }
    }
}
