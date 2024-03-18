// fn main() {
//         tonic_build::compile_protos("proto/orderbook.proto")
//             .unwrap_or_else(|e| panic!("Failed to compile protos {:?}", e));
//     }

use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
   let proto_file = "./proto/orderbook.proto";
   let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

   tonic_build::configure()
           .protoc_arg("--experimental_allow_proto3_optional") // for older systems
           .build_client(true)
           .build_server(true)
           .file_descriptor_set_path(out_dir.join("store_descriptor.bin"))
           .compile(&[proto_file], &["proto"])?;

   Ok(())
}
