use crate::db::establish_connection;
use crate::log_grpc_request;
use crate::modules::domain_records::dtos::CreateDomainRecordDto as InternalCreateDomainRecordDto;
use crate::modules::domain_records::model::{save_domain_record, DomainRecord};
use crate::proto::{
    domain_record_service_server::DomainRecordService, CreateDomainRecordDto,
    DeleteDomainRecordDto, GetRecordByIdDto, RpcDomainRecord, UpdateDomainRecordDto,
};
use crate::utils::reload_bind9_server;
use tonic::{Request, Response, Status};

#[derive(Debug, Default)]
pub struct DomainRecordServiceImpl {}

#[tonic::async_trait]
impl DomainRecordService for DomainRecordServiceImpl {
    async fn create_domain_record(
        &self,
        request: Request<CreateDomainRecordDto>,
    ) -> Result<Response<RpcDomainRecord>, Status> {
        let input = request.into_inner();
        log_grpc_request!("DomainRecordService", "create_domain_record", &input);

        // Map proto DTO to internal DTO
        let dto = InternalCreateDomainRecordDto {
            ttl: if input.ttl == 0 {
                None
            } else {
                Some(input.ttl)
            },
            class: input.class,
            host: input.host.clone(),
            record_type: input.record_type,
            mx_preference: if input.mx_preference == 0 {
                None
            } else {
                Some(input.mx_preference)
            },
            domain_id: Some(input.domain_id),
            value: input.value,
            soa_host_mail: None,
            soa_serial_number: None,
            soa_refresh_time: None,
            soa_retry_time: None,
            soa_expire_time: None,
            soa_negative_cache_ttl: None,
        };

        let result = match save_domain_record(dto, input.domain_id) {
            Ok(saved_record) => {
                // Update SOA serial number after creating the record
                if let Err(e) = crate::utils::update_soa_serial_for_domain(input.domain_id) {
                    eprintln!("Failed to update SOA serial: {}", e);
                }

                let rpc_record = RpcDomainRecord {
                    id: saved_record.id,
                    host: saved_record.host,
                    r#type: saved_record.record_type,
                    value: saved_record.value,
                    priority: saved_record.mx_preference.unwrap_or(3600),
                    ttl: saved_record.ttl.unwrap_or(0),
                };
                Ok(Response::new(rpc_record))
            }
            Err(e) => Err(Status::internal(format!("Failed to save record: {}", e))),
        };
        reload_bind9_server();
        result
    }

    async fn update_domain_record(
        &self,
        request: Request<UpdateDomainRecordDto>,
    ) -> Result<Response<RpcDomainRecord>, Status> {
        let input = request.into_inner();
        use crate::schema::domain_records::dsl::*;
        use diesel::prelude::*;

        let mut conn = crate::db::establish_connection();

        // Find the record
        let record_result = domain_records
            .find(input.id)
            .first::<DomainRecord>(&mut conn);

        let result = match record_result {
            Ok(found_record) => {
                // Update the record in the database
                let updated_record = diesel::update(domain_records.filter(id.eq(input.id)))
                    .set((
                        host.eq(&input.host),
                        value.eq(&input.value),
                        ttl.eq(Some(&input.ttl)),
                        record_type.eq(&input.r#type),
                    ))
                    .get_result::<DomainRecord>(&mut conn)
                    .expect("Failed to update record");

                // Regenerate the zone file to ensure proper formatting and SOA serial update
                if let Err(e) = crate::utils::regenerate_zone_file(found_record.domain_id.unwrap())
                {
                    eprintln!("Failed to regenerate zone file: {}", e);
                }

                // Return the updated record
                let rpc_record = RpcDomainRecord {
                    id: updated_record.id,
                    host: updated_record.host,
                    value: updated_record.value,
                    priority: updated_record.mx_preference.unwrap_or(3600),
                    ttl: updated_record.ttl.unwrap_or(3600),
                    r#type: updated_record.record_type,
                };
                Ok(Response::new(rpc_record))
            }
            Err(e) => Err(Status::not_found(format!("Record not found: {}", e))),
        };
        reload_bind9_server();
        result
    }

    async fn delete_domain_record(
        &self,
        request: Request<DeleteDomainRecordDto>,
    ) -> Result<Response<RpcDomainRecord>, Status> {
        let input = request.into_inner();
        use crate::schema::domain_records::dsl::*;
        use diesel::prelude::*;

        let mut conn = crate::db::establish_connection();

        // Find the record
        let record_result = domain_records
            .find(input.id)
            .first::<DomainRecord>(&mut conn);

        let result = match record_result {
            Ok(found_record) => {
                // Soft delete: set is_deleted = true
                let _ = diesel::update(domain_records.filter(id.eq(input.id)))
                    .set(is_deleted.eq(true))
                    .execute(&mut conn);

                // Regenerate the zone file to ensure proper formatting and SOA serial update
                if let Err(e) = crate::utils::regenerate_zone_file(found_record.domain_id.unwrap())
                {
                    eprintln!("Failed to regenerate zone file: {}", e);
                }

                // Return the deleted record
                let rpc_record = RpcDomainRecord {
                    id: found_record.id,
                    host: found_record.host,
                    value: found_record.value,
                    priority: found_record.mx_preference.unwrap_or(3600),
                    ttl: found_record.ttl.unwrap_or(3600),
                    r#type: found_record.class,
                };
                Ok(Response::new(rpc_record))
            }
            Err(e) => Err(Status::not_found(format!("Record not found: {}", e))),
        };
        reload_bind9_server();
        result
    }

    async fn get_record_by_id(
        &self,
        request: Request<GetRecordByIdDto>,
    ) -> Result<Response<RpcDomainRecord>, Status> {
        use crate::schema::domain_records::dsl::*;
        use diesel::prelude::*;

        let mut conn = establish_connection();
        let input = request.into_inner();

        let record_result = domain_records
            .find(input.id)
            .first::<DomainRecord>(&mut conn);

        match record_result {
            Ok(record) => {
                let retrieved_domain_record = RpcDomainRecord {
                    id: record.id,
                    value: record.value,
                    host: record.host,
                    ttl: record.ttl.unwrap_or(3600),
                    priority: record.mx_preference.unwrap_or(1000),
                    r#type: record.record_type,
                };

                Ok(Response::new(retrieved_domain_record))
            }
            Err(e) => Err(Status::not_found(format!("Record not found: {}", e))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log_grpc_request;
    use crate::utils::generate_incremental_zone_serial;
    use chrono::Datelike;

    #[test]
    fn test_serial_number_generation() {
        // Test that serial numbers are generated in the correct format
        let domain = "ikimodoka.coop.rw";

        let serial = generate_incremental_zone_serial(domain).unwrap();

        // Serial should be 10 characters: YYYYMMDDXX
        assert_eq!(serial.len(), 10);

        // Should start with today's date
        let now = chrono::Utc::now();
        let today_prefix = format!("{:04}{:02}{:02}", now.year(), now.month(), now.day());
        assert!(serial.starts_with(&today_prefix));

        // Last two characters should be a number
        let increment = &serial[8..10];
        assert!(increment.parse::<u8>().is_ok());
    }

    #[test]
    fn test_serial_number_increment() {
        let domain = "ikimodoka.coop.rw";
        let today = chrono::Utc::now();
        let today_prefix = format!("{:04}{:02}{:02}", today.year(), today.month(), today.day());

        // Test first serial of the day
        let first_serial = format!("{}01", today_prefix);
        let next_serial = generate_incremental_zone_serial(domain).unwrap();

        // If it's the same day, should increment
        if next_serial.starts_with(&today_prefix) {
            let current_increment: u8 = first_serial[8..10].parse().unwrap();
            let next_increment: u8 = next_serial[8..10].parse().unwrap();
            assert!(next_increment >= current_increment);
        }
    }
}
