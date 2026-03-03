pub mod traits;
use chrono::{Datelike, Utc};
use diesel::{BoolExpressionMethods, QueryDsl, RunQueryDsl};
use dotenv::dotenv;
use std::{env, fmt::format, path::Path};

use crate::modules::domain_records::{DNSRecordType, Record, SOAProperties};

pub fn resolve_zonefile_path(domain: &str) -> String {
    return format!("{}/{}/db.{}", config().BIND9_PATH, domain, domain);
}

pub fn generate_incremental_zone_serial(
    domain: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let zone_path = resolve_zonefile_path(&domain.to_string());
    let now = Utc::now();
    let today_prefix = format!("{:04}{:02}{:02}", now.year(), now.month(), now.day());

    if Path::new(&zone_path).exists() {
        let content = std::fs::read_to_string(&zone_path)?;

        if let Some(current_serial) = extract_serial_from_zonefile(&content) {
            if current_serial.starts_with(&today_prefix) {
                // Same day - increment
                if current_serial.len() >= 10 {
                    let current_increment: u8 = current_serial[8..10].parse().unwrap_or(0);
                    let new_increment = (current_increment + 1).min(99);
                    let new_serial = format!("{}{:02}", today_prefix, new_increment);
                    return Ok(new_serial);
                }
            }
        }
    }

    // New day or no existing file
    Ok(format!("{}01", today_prefix))
}

fn extract_serial_from_zonefile(content: &str) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Look for SOA record lines that contain the opening parenthesis
        if trimmed.contains("SOA") && trimmed.contains('(') {
            // The serial number is on the next line after the opening parenthesis
            if i + 1 < lines.len() {
                let serial_line = lines[i + 1].trim();
                // Extract just the digits for the serial number
                let serial: String = serial_line
                    .chars()
                    .take_while(|c| c.is_ascii_digit())
                    .collect();

                if !serial.is_empty() {
                    return Some(serial);
                }
            }
        }

        // Fallback: Look for lines that are just numbers (for backward compatibility)
        if trimmed.len() >= 10 && trimmed.len() <= 12 && trimmed.chars().all(|c| c.is_ascii_digit())
        {
            return Some(trimmed.to_string());
        }
    }

    None
}

pub fn reload_bind9_server() {
    // This function reloads the BIND9 server by executing the appropriate system command.
    // It uses the `rndc reload` command, which is the standard way to reload BIND9 configuration and zones.
    // Any errors during execution are printed to stderr.
    // use std::process::Command;

    // let output = Command::new("rndc").arg("reload").output();

    // match output {
    //     Ok(result) => {
    //         if !result.status.success() {
    //             // Command executed but returned a non-zero exit code
    //             eprintln!(
    //                 "Failed to reload BIND9 server: {}",
    //                 String::from_utf8_lossy(&result.stderr)
    //             );
    //         } else {
    //             // Successfully reloaded
    //             println!("BIND9 server reloaded successfully.");
    //         }
    //     }
    //     Err(e) => {
    //         // Failed to execute the command itself
    //         eprintln!("Error executing rndc reload: {}", e);
    //     }
    // }

    println!("Reloading BIND9 server");
}

pub fn update_soa_serial_for_domain(
    record_domain_id: i32,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::db::establish_connection;
    use crate::schema::domain_records::dsl::*;
    use diesel::prelude::*;

    let mut conn = establish_connection();

    // First, find the domain name for this domain_id
    let domain_name = {
        use crate::schema::domains::dsl::*;
        domains
            .filter(id.eq(record_domain_id))
            .select(domain)
            .first::<String>(&mut conn)?
    };

    // Generate new serial number
    let new_serial = generate_incremental_zone_serial(&domain_name)?;

    // Update SOA record with new serial number
    // Fix: Use .eq(Some(domain_id)) since domain_id column is nullable
    diesel::update(
        domain_records
            .filter(domain_id.eq(Some(record_domain_id)))
            .filter(record_type.eq("SOA")),
    )
    .set(soa_serial_number.eq(new_serial.parse::<i64>()?))
    .execute(&mut conn)?;

    println!(
        "Updated SOA serial to {} for domain: {}",
        new_serial, domain_name
    );
    Ok(())
}

pub fn regenerate_zone_file(record_domain_id: i32) -> Result<(), Box<dyn std::error::Error>> {
    use crate::db::establish_connection;
    use crate::modules::domain_records::{GenFromDomainrecord, Record};
    use crate::schema::domains;
    use crate::utils::traits::Print;
    use diesel::ExpressionMethods;
    use diesel::Queryable;

    let mut conn = establish_connection();

    // Get domain name
    let domain_name = {
        use crate::schema::domains::dsl::*;
        domains
            .filter(id.eq(record_domain_id))
            .select(domain)
            .first::<String>(&mut conn)?
    };

    // Update SOA serial number first
    update_soa_serial_for_domain(record_domain_id)?;

    // Get all records for this domain
    // Fix: Use .eq(Some(domain_id)) since domain_id column is nullable
    let domain_records = {
        use crate::schema::domain_records::dsl::*;
        domain_records
            .filter(domain_id.eq(Some(record_domain_id)))
            .filter(is_deleted.eq(Some(false)).or(is_deleted.is_null()))
            .load::<crate::modules::domain_records::model::DomainRecord>(&mut conn)?
    };

    // Convert to Record objects and generate zone file content
    let mut zone_content = format!(
        "$TTL {}\n$ORIGIN {}.\n\n",
        config().DEFAULT_TTL,
        domain_name
    );

    for db_record in domain_records {
        let record = Record::from_domain_record(&db_record);
        zone_content.push_str(&record.print());
    }

    // Write to zone file
    let zone_file_path = resolve_zonefile_path(&domain_name);
    std::fs::write(&zone_file_path, zone_content)?;

    // Reload BIND9
    reload_bind9_server();

    println!("Regenerated zone file for domain: {}", domain_name);
    Ok(())
}

pub struct Env {
    pub DNS_ADDRESS: String,
    pub DATABASE_URL: String,     // The database connection string
    pub BIND9_PATH: String,       // Directory where zone files are stored
    pub BIND9_CONFIGFILE: String, // Path to the named.conf.local file
    pub DEFAULT_TTL: u32,         // Default TTL value for DNS records
    pub HOSTMASTER_EMAIL: String, // Hostmaster email address for SOA records
    pub DNS_HOSTNAME: String,     // Hostmaster email address for SOA records
}

pub fn config() -> Env {
    dotenv().ok();

    // Load environment variables using std::env::var and parse as needed.
    // If a variable is missing, panic with a clear message.
    let env = Env {
        DNS_ADDRESS: std::env::var("DNS_ADDRESS").expect("DNS ADDRESS not set"),
        DATABASE_URL: std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
        BIND9_PATH: std::env::var("BIND9_PATH").expect("BIND9_PATH must be set"),
        BIND9_CONFIGFILE: std::env::var("BIND9_CONFIGFILE").expect("BIND9_CONFIGFILE must be set"),
        DEFAULT_TTL: std::env::var("DEFAULT_TTL")
            .expect("DEFAULT_TTL must be set")
            .parse::<u32>()
            .expect("DEFAULT_TTL must be a valid u32"),
        HOSTMASTER_EMAIL: std::env::var("HOSTMASTER_EMAIL").expect("HOSTMASTER_EMAIL must be set"),
        DNS_HOSTNAME: std::env::var("DNS_HOSTNAME").expect("DNS_HOSTNAME must be set"),
    };

    env
}
