use std::env;

use crate::{
    log_grpc_error, log_grpc_request, log_grpc_response,
    modules::{
        domain_records::{
            dtos::{CreateDomainRecordDto, InitializeFromRecord},
            model::save_domain_record,
        },
        domains::model::add_domain,
    },
    proto::{CreateDomainDto, DeleteDomainRequest, DeleteDomainResponse, RpcDomain},
};

use crate::{modules::domains::model::Domain, utils::traits::Print};

use tonic::{Request, Response, Status};

use crate::{
    modules::domains::DomainConfig,
    proto::{self, domain_service_server::DomainService},
    utils::traits::Initialize,
};

use crate::modules::domains::model::{self};

#[derive(Debug, Default)]
pub struct DomainsServiceImpl {}

#[tonic::async_trait]
impl DomainService for DomainsServiceImpl {
    async fn create_domain(
        &self,
        request: Request<proto::CreateDomainDto>,
    ) -> Result<tonic::Response<proto::RpcDomain>, Status> {
        let input = request.get_ref();
        log_grpc_request!("DomainService", "create_domain", input);

        let mut conn = crate::db::establish_connection();

        // Query the domains table to find the whole domain struct by its domain name
        // This will return the first Domain that matches the given domain name.
        if let Ok(existing_domain) = domains
            .filter(domain.eq(&input.domain))
            .first::<Domain>(&mut conn)
        {
            // If the domain already exists, return it as an RpcDomain response
            let response = proto::RpcDomain {
                id: existing_domain.id.to_string(),
                default_ttl: env::var("DEFAULT_TTL")
                    .unwrap_or_default()
                    .parse::<i64>()
                    .unwrap_or(3600), // You may want to fetch this from config or DB if needed
                domain: existing_domain.domain,
            };
            return Ok(tonic::Response::new(response));
        }

        let domain_config = DomainConfig::new(&input.domain, &input.ip_address)
            .expect("Setting up default domain records failed");

        println!("{}", domain_config.print());

        // Find the domain by its domain name (input.domain) instead of by id.
        // This will return the first domain that matches the given domain name.
        use crate::schema::domains::dsl::*;
        use diesel::prelude::*;

        // Establish a database connection

        let saved_domain = add_domain(CreateDomainDto {
            domain: domain_config.domain,
            ip_address: input.ip_address.clone(),
        })
        .expect("An error occured while saving the domains");

        for record in domain_config.records {
            save_domain_record(CreateDomainRecordDto::from_record(record), saved_domain.id)
                .expect("Error occured while saving the domain configs");
        }

        let response = RpcDomain {
            id: saved_domain.id.to_string(),
            default_ttl: 3600,
            domain: saved_domain.domain,
        };

        log_grpc_response!("DomainService", "create_domain", &response);
        Ok(tonic::Response::new(response))
    }

    async fn delete_domain(
        &self,
        request: Request<DeleteDomainRequest>,
    ) -> Result<Response<DeleteDomainResponse>, Status> {
        let input = request.get_ref();
        log_grpc_request!("DomainService", "delete_domain", input);

        let domain_id = request.into_inner().domain_id;

        // Delete from database and handle named.conf
        let response = model::delete_domain(domain_id);

        let delete_response = DeleteDomainResponse {
            success: response.success,
            message: response.message,
            errors: response.errors.unwrap_or_default(),
        };

        log_grpc_response!("DomainService", "delete_domain", &delete_response);
        Ok(Response::new(delete_response))
    }
}
