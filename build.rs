fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    tonic_build::configure()
        .file_descriptor_set_path(format!("{}/dns_scriptylabs_com.bin", out_dir))
        .compile(
            &[
                "proto/domain_records_grpc.proto",
                "proto/domains_grpc.proto",
            ],
            &["proto"],
        )?;
    Ok(())
}
