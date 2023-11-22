
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .out_dir("./src/generated/")
        .compile(
            &["proto/messages.proto","proto/services.proto"], 
            &["protos/proto"]
        )?;
    Ok(())
}
