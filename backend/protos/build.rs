// fn main() {
//     // Watch banking-proto directories for changes. We don't explicitly compile every banking-proto, but they might
//     // be included transitively from other protos. Watching the whole directories catches
//     // these transitive changes.
//     let rebuild_watches = &["protos", "build.rs"];
//     for rebuild_watch in rebuild_watches {
//         println!("cargo:rerun-if-changed={}", rebuild_watch);
//     }

//     let proto_files = &["protos/database.proto", "protos/util.proto"];

//     // Should do this, but it drives VSCode nuts re-running rust-analyzer constantly.
//     // Need to figure that out. Maybe move generated outside of src/.
//     // let path = "src/generated";
//     // fs::remove_dir_all(path).unwrap();
//     // fs::create_dir(path).unwrap();

//     tonic_build::configure()
//         .build_server(true)
//         .build_client(true)
//         .protoc_arg("-I./protos")
//         .type_attribute(".", "#[derive(serde::Serialize,serde::Deserialize)]")
//         .extern_path(".google.protobuf.Any", "::prost_wkt_types::Any")
//         .extern_path(".google.protobuf.Timestamp", "::prost_wkt_types::Timestamp")
//         .extern_path(".google.protobuf.Value", "::prost_wkt_types::Value")
//         .include_file("mod.rs")
//         .out_dir("./src/generated/")
//         .compile(proto_files, &["."])
//         .unwrap_or_else(|e| panic!("protobuf compile error: {}", e));
// }

// build.rs

// build.rs

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
