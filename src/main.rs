mod db;
mod dns_response;
mod middleware;
mod modules;
mod schema;
mod types;
mod utils;

mod proto {
    tonic::include_proto!("domain_records_grpc");
    tonic::include_proto!("domains_grpc");

    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("dns_scriptylabs_com");
    use tonic::transport::Server;
    use tower::ServiceBuilder;

    use crate::middleware::logging::LoggingLayer;
    use crate::modules::domain_records::service::DomainRecordServiceImpl;
    use crate::proto::domain_record_service_server::DomainRecordServiceServer;
    use crate::{
        modules::domains::service::DomainsServiceImpl,
        proto::domain_service_server::DomainServiceServer, utils::config,
    };

    pub async fn run_grpc_server() -> Result<(), Box<dyn std::error::Error>> {
        // Initialize tracing/logging
        init_logging();

        let addr: std::net::SocketAddr = config().DNS_ADDRESS.parse()?;

        tracing::info!("Starting gRPC server on {}", addr);

        let reflection_service = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
            .build()?;

        // Create services
        let domain_service = DomainServiceServer::new(DomainsServiceImpl::default());
        let domain_record_service =
            DomainRecordServiceServer::new(DomainRecordServiceImpl::default());

        // Build server with logging middleware
        Server::builder()
            .layer(
                ServiceBuilder::new()
                    .layer(LoggingLayer::new())
                    .into_inner(),
            )
            .add_service(reflection_service)
            .add_service(domain_service)
            .add_service(domain_record_service)
            .serve(addr)
            .await?;

        Ok(())
    }

    fn init_logging() {
        use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

        // Initialize tracing with environment filter support
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                    "dns_scriptylabs_com=info,tower_http=debug,tonic=debug".into()
                }),
            )
            .with(tracing_subscriber::fmt::layer().with_target(true))
            .init();

        tracing::info!("Logging initialized");
    }
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    proto::run_grpc_server().await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        modules::{
            domain_records::{DNSRecordType, Record, SOAProperties},
            domains::DomainConfig,
        },
        utils::traits::{Defaults, Initialize, Print},
    };

    #[test]
    fn test_change_record_to_string() {
        // Create domain config and handle potential errors
        let domain = DomainConfig::new(&String::from("bellamy.rw"), &String::from("76.76.21.21"))
            .expect("Failed to create domain config");
        println!("{}", domain.print());

        assert_eq!("", "");
    }
}
