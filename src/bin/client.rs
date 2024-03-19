use jsonrpsee::core::client::ClientT;
use jsonrpsee::http_client::HttpClientBuilder;
use jsonrpsee::rpc_params;

use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

#[tokio::main]
async fn main() {
    let url = format!("http://{}", "127.0.0.1:9997");
    let client = HttpClientBuilder::default().build(url).unwrap();
    let working_directory = Path::new("/Users/mitch/apps/acvm_server/fixtures");

    // read the witness file
    let witness_file_name = "witnessMap.bin".to_string();
    let mut file = File::open(working_directory.join(witness_file_name)).unwrap();
    let mut witness_map = Vec::new();
    file.read_to_end(&mut witness_map).unwrap();

    // read bytecode file
    let bytecode_file_name = "bytecode".to_string();
    let mut file = File::open(working_directory.join(bytecode_file_name)).unwrap();
    let mut bytecode = Vec::new();
    file.read_to_end(&mut bytecode).unwrap();

    // run it
    let params = rpc_params![witness_map, bytecode];
    let response: Result<String, _> = client.request("run", params).await;

    match response {
        Ok(witness) => {
            // write the bin to the fixtures directory
            let output_file_name = working_directory.join("output_witness.bin");
            let mut file = File::create(output_file_name).unwrap();

            match file.write_all(witness.as_bytes()) {
                Ok(_) => println!("Success!"),
                Err(e) => println!("Error: {:?}", e),
            }
        }
        Err(e) => println!("Error: {:?}", e),
    }
}

// tests
#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io::{Read, Write},
        path::Path,
    };

    use acir::native_types::WitnessMap;

    use acvm_server::errors::{CliError, FilesystemError};

    #[test]
    fn test_read_inputs_from_file() {
        let working_directory = Path::new("/Users/mitch/apps/acvm_server/fixtures");
        let file_name = "witnessMap.toml".to_string();
        let result =
            acvm_server::utils::read_inputs_from_file(working_directory, &file_name).unwrap();

        let bin = Vec::try_from(result).unwrap();
        // write the bin to the fixtures directory
        let output_file_name = working_directory.join("witnessMap.bin");
        let mut file = File::create(output_file_name).unwrap();
        file.write_all(&bin).unwrap();
    }

    #[test]
    fn test_read_outputs_from_file() {
        let working_directory = Path::new("/Users/mitch/apps/acvm_server/fixtures");
        let file_name = "output_witness.bin".to_string();
        // read file
        let mut file = File::open(working_directory.join(file_name)).unwrap();
        let mut witness_map = Vec::new();
        file.read_to_end(&mut witness_map).unwrap();
        let _ = WitnessMap::try_from(witness_map.as_slice()).unwrap();
    }
}
