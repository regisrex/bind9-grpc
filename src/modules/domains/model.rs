use crate::{
    dns_response::DnsResponse,
    proto::CreateDomainDto,
    utils::{config, traits::Print},
};
use diesel::{result::Error, ExpressionMethods, QueryDsl, RunQueryDsl};
use std::fs::OpenOptions;
use std::{
    fs::{read_to_string, write},
    io::Write,
};

use crate::{db::establish_connection, schema::domains};
use dotenv::dotenv;
use std::env;

use diesel::prelude::{Identifiable, Insertable, Queryable};

#[derive(Queryable, Identifiable, PartialEq, Debug)]
#[diesel(table_name = domains)]
pub struct Domain {
    pub id: i32,
    pub domain: String,
    pub is_deleted: Option<bool>,
}

impl Print for Domain {
    fn print(&self) -> String {
        // Fixed self reference syntax
        let mut domain_string_value = String::new(); // Using String::new() is more idiomatic
        dotenv().ok();
        let default_ttl =
            env::var("DEFAULT_TTL").expect("Invalid .env config, missing : DEFAULT_TTL");

        domain_string_value.push_str(&format!(
            "\n\n$TTL {}\n$ORIGIN {}.\n\n",
            default_ttl, self.domain
        ));

        domain_string_value
    }
}

#[derive(Insertable)]
#[diesel(table_name = domains)]
pub struct NewDomain {
    pub domain: String,
    pub is_deleted: bool,
}

pub fn add_domain(data: CreateDomainDto) -> Result<Domain, Error> {
    use crate::schema::domains::dsl::*;
    let mut conn = establish_connection();
    let domain_str = data.domain.clone();

    let bind_config_file = config().BIND9_CONFIGFILE;

    // Ensure BIND9 config file directory exists
    if let Some(parent) = std::path::Path::new(&bind_config_file).parent() {
        std::fs::create_dir_all(parent).expect("Failed to create BIND9 config directory");
    }

    let new_domain = NewDomain {
        is_deleted: false,
        domain: data.domain.clone(),
    };
    let inserted_domain: Domain = diesel::insert_into(domains)
        .values(&new_domain)
        .get_result(&mut conn)?;

    // Ensure base BIND9 directory exists
    std::fs::create_dir_all(&config().BIND9_PATH).expect("Failed to create base BIND9 directory");

    // Create directory structure for this domain
    let bind_dir = format!("{}/{}", config().BIND9_PATH, new_domain.domain);
    std::fs::create_dir_all(&bind_dir).expect("Failed to create bind directory structure");

    // Create zone configuration file path
    let zone_config_file_path = format!("{}/zone.conf", bind_dir);
    // Create zone records file path
    let zone_records_file_path = format!("{}/db.{}", bind_dir, new_domain.domain);

    // Create zone configuration content
    let zone_config_content = format!(
        "zone \"{}\" {{\n\
         \ttype master;\n\
         \tfile \"{}\";\n\
         \tallow-update {{ none; }};\n\
         }};\n",
        new_domain.domain, zone_records_file_path
    );

    // Write zone configuration file
    let mut zone_config_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&zone_config_file_path)
        .expect("Error occurred while creating zone config");

    writeln!(zone_config_file, "{}", zone_config_content)
        .expect("Error occurred while writing zone config");

    // Create zone records file with initial content
    let mut zone_records_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&zone_records_file_path)
        .expect("Error occurred while creating zone records file");

    // Write initial zone file content with TTL and ORIGIN
    writeln!(
        zone_records_file,
        "$TTL {}\n$ORIGIN {}.\n",
        env::var("DEFAULT_TTL").unwrap_or_else(|_| "3600".to_string()),
        new_domain.domain
    )
    .expect("Error occurred while writing initial zone records");

    let mut bind_config_file_config = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(&bind_config_file)
        .expect("Error occurred while creating zone config");

    writeln!(
        bind_config_file_config,
        "include \"{}\";",
        zone_config_file_path,
    )
    .expect("Error occurred while linking the new domain record to named config.");
    Ok(inserted_domain)
}

pub fn delete_domain(domain_id: i32) -> DnsResponse<Domain> {
    use crate::schema::domains::dsl::*;
    let mut conn = establish_connection();

    // First check if domain exists
    let domain_result = domains.find(domain_id).first::<Domain>(&mut conn);

    match domain_result {
        Ok(deleted_domain) => {
            // Domain exists, proceed with soft deletion
            let updated = diesel::update(domains.filter(id.eq(domain_id)))
                .set(is_deleted.eq(true))
                .execute(&mut conn);

            match updated {
                Ok(_) => {
                    // Remove include from named.conf
                    let bind_config_file = config().BIND9_CONFIGFILE;
                    let zone_config_path = format!(
                        "{}/{}/zone.conf",
                        config().BIND9_PATH,
                        deleted_domain.domain
                    );
                    let include_line = format!("include \"{}\";", zone_config_path);

                    if let Ok(content) = read_to_string(&bind_config_file) {
                        let new_content = content.replace(&include_line, "");
                        if let Err(e) = write(&bind_config_file, new_content) {
                            eprintln!("Error updating named.conf: {}", e);
                        }
                    } else {
                        // If file doesn't exist or can't be read, create an empty one
                        if let Err(e) = write(&bind_config_file, "") {
                            eprintln!("Error creating named.conf: {}", e);
                        }
                    }

                    DnsResponse {
                        success: true,
                        message: "Domain marked as deleted successfully".to_string(),
                        data: Some(deleted_domain),
                        errors: None,
                    }
                }
                Err(e) => DnsResponse {
                    success: false,
                    message: "Failed to mark domain as deleted".to_string(),
                    data: None,
                    errors: Some(vec![e.to_string()]),
                },
            }
        }
        Err(e) => DnsResponse {
            success: false,
            message: "Domain not found".to_string(),
            data: None,
            errors: Some(vec![e.to_string()]),
        },
    }
}

pub fn get_domain(domain_id: i32) -> Result<Domain, Error> {
    use crate::schema::domains::dsl::*;
    let mut conn = establish_connection();

    domains
        .find(domain_id)
        .first::<Domain>(&mut conn)
        .map_err(|e| Error::NotFound)
}
