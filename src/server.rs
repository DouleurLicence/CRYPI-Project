use tonic::transport::{Certificate, Identity, Server, ServerTlsConfig};
use tonic::{Request, Response, Status};

use ring::digest::{Context, Digest, SHA256};

use file::file_server::{File, FileServer};
use file::{FileFinished, FileResponse, FileTransfer};

use std::sync::Mutex;

use std::io::Write;

use sha2::Sha256;
use hmac::{Hmac, Mac};
use hex_literal::hex;

mod csv_file;

// Import the generated proto-rust file into a module
pub mod file {
    tonic::include_proto!("file");
}

// Implement the service skeleton for the "File" service
// defined in the proto
#[derive(Debug, Default)]
pub struct MyServer {
    received_data: Mutex<Vec<u8>>,
}

// Implement the service function(s) defined in the proto
// for the File service (SendFile...)
#[tonic::async_trait]
impl File for MyServer {
    async fn send_file(
        &self,
        request: Request<FileTransfer>,
    ) -> Result<Response<FileResponse>, Status> {
        let request_contents = request.into_inner();
        let file_contents = request_contents.content;
        let client_hash = request_contents.hash; // Assuming the client sends the hash along with the content

        // Before appending the received chunk to received_data,
        // there is a checkup of its integrity using cryptographic
        // hash functions (in this case SHA-256)

        // Compute the hash of the received chunk
        let mut context = Context::new(&SHA256);
        context.update(&file_contents);
        let computed_hash = context.finish();

        // Verify the integrity of the chunk by comparing the computed hash with the received hash
        if computed_hash.as_ref() != client_hash.as_slice() {
            return Err(Status::invalid_argument(
                "Hash mismatch, data integrity compromised",
            ));
        }

        // Append received chunk to received_data
        self.received_data.lock().unwrap().extend(file_contents);

        let response = file::FileResponse {
            message: format!("OK").into(),
        };

        Ok(Response::new(response))
    }

    async fn finish_transfer(
        &self,
        request: Request<FileFinished>,
    ) -> Result<Response<FileResponse>, Status> {
        // Get the filename
        let filename = request.into_inner().filename;

        // Deserialize the received data
        let mut received_data = self.received_data.lock().unwrap();
        let records: Vec<csv_file::Record> = bincode::deserialize(&*received_data).unwrap();
        println!("{} received and saved.", filename);

        // Clear the received_data for future transfers
        received_data.clear();

        let response = file::FileResponse {
            message: format!("OK").into(),
        };
        Ok(Response::new(response))
    }

    async fn priming_send(
        &self,
        request: Request<file::FileRequest>,
    ) -> Result<Response<file::FileResponse>, Status> {
        // Get the filename from the request
        let filename = request.into_inner().filename;

        // Create a file with the filename
        let mut file = std::fs::File::create(&filename)?;

        match file.write_all(b"") {
            Ok(_) => {
                let response = file::FileResponse {
                    message: format!("OK").into(),
                };

                Ok(Response::new(response))
            }
            Err(_) => {
                let response = file::FileResponse {
                    message: format!("ERROR").into(),
                };

                Ok(Response::new(response))
            }
        }
    }
}

// Runtime to run our server
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get the args passed to the program
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        println!("Usage: {} <port>", args[0]);
        return Ok(());
    }

    let cert = std::fs::read_to_string("server.crt")?;
    let key = std::fs::read_to_string("server.key")?;
    let server_identity = Identity::from_pem(cert, key);

    let client_ca_cert = std::fs::read_to_string("ca.crt")?;
    let client_ca_cert = Certificate::from_pem(client_ca_cert);

    // Consider implementing certificate pinning with `rustls` by providing
    // a custom certificate verifier that checks the server's certificate
    // against a known, trusted copy.

    let addr = "127.0.0.1:".to_owned() + &args[1];
    let addr = addr.parse()?;
    let server = MyServer::default();

    let tls = ServerTlsConfig::new()
        .identity(server_identity)
        .client_ca_root(client_ca_cert);

    println!("Hosting on port {}, waiting for commands...", args[1]);
    Server::builder()
        .tls_config(tls)?
        .add_service(FileServer::new(server))
        .serve(addr)
        .await?;

    Ok(())
}
