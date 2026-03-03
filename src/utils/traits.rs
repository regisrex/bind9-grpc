use std::io::Error;

use tonic::Request;

use crate::modules::domains::DomainConfig;

pub trait Print {
    fn print(self: &Self) -> String {
        "To be implemented".to_string()
    }
}

pub trait PrintableToZone {
    fn print_to_zonefile(self: &Self) -> Result<(), Error>;
}

pub trait Initialize {
    fn new(domain_name: &String, ip_address : &String) -> Result<DomainConfig, Error>;
}

pub trait Defaults {
    fn defaults(domain_name: String) -> Self
    where
        Self: Sized;
}
