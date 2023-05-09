use ndarray::{Array1, ArrayBase};
use tonic::transport::{Certificate, Identity, Server, ServerTlsConfig};
use tonic::{Request, Response, Status};

use ring::digest::{Context, SHA256};

use bincode::deserialize;

use file::file_server::{File, FileServer};
use file::{FileFinished, FileResponse, FileTransfer};

use std::sync::Mutex;

use std::io::Write;

use hmac::{Hmac, Mac};
use sha2::Sha256;

// Create alias for HMAC-SHA256
type HmacSha256 = Hmac<Sha256>;

mod csv_file;
mod normalize;
mod training;

// Import the generated proto-rust file into a module
pub mod file {
    tonic::include_proto!("file");
}

// Implement the service skeleton for the "File" service
// defined in the proto
#[derive(Debug, Default)]
pub struct MyServer {
    received_data: Mutex<Vec<u8>>,
    coefs_path: Mutex<String>,
    training_file: Mutex<String>,
    prediction_file: Mutex<String>,
    hmac_hash: Mutex<Vec<u8>>,
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
        let request_contents = request.into_inner();
        // Get the filename
        let filename = request_contents.filename;
        let received_hmac_hash = request_contents.hmac_hash;

        println!("{} received", filename);
        println!("Checking integrity...");

        let mut received_data = self.received_data.lock().unwrap();

        // Create a new HMAC instance for this file transfer
        let mut hmac =
            HmacSha256::new_from_slice(b"secret").expect("HMAC can take key of any size");

        // Compute the HMAC hash of the received data
        hmac.update(&received_data);
        let mut computed_hmac_hash = hmac.clone().finalize().into_bytes().to_vec();

        // Verify the integrity of the entire file by comparing the computed HMAC hash with the received HMAC hash
        if computed_hmac_hash != received_hmac_hash {
            return Err(Status::invalid_argument(
                "HMAC hash mismatch, data integrity compromised",
            ));
        }

        println!("Integrity OK");

        // if it is a csv file then deserialize it and write it to a csv file
        if filename.ends_with(".csv") {
            // Deserialize the received data into a vector of CSV records
            let content_deserialized = deserialize::<Vec<csv_file::Record>>(&*received_data)
                .expect("Failed to deserialize");

            // Write the deserialized data to a CSV file
            csv_file::write_csv_file(content_deserialized, &filename);
        }

        if filename.ends_with(".txt") {
            // Deserialize the received data into a vector of CSV records
            let content_deserialized =
                deserialize::<Array1<f64>>(&*received_data).expect("Failed to deserialize");

            // Write the deserialized data to a txt file
            csv_file::write_array1_to_file(&content_deserialized, &filename);
        }

        // Clear the received_data for future transfers
        received_data.clear();
        computed_hmac_hash.clear();

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
        let request_contents = request.into_inner();
        let filename = request_contents.filename;
        let training = request_contents.train;
        let coefs = request_contents.coefs;

        // Saving of whatever type the client sends
        if training {
            self.training_file
                .lock()
                .unwrap()
                .push_str(&filename.to_string());
        } else {
            if coefs {
                self.coefs_path
                    .lock()
                    .unwrap()
                    .push_str(&filename.to_string());
            } else {
                self.prediction_file
                    .lock()
                    .unwrap()
                    .push_str(&filename.to_string());
            }
        }

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

    async fn launch_training(
        &self,
        _request: Request<file::RequestTraining>,
    ) -> Result<Response<file::ResponseAccuracy>, Status> {
        let mut message = String::from("");
        let mut accuracy = 0.0;
        if self.training_file.lock().unwrap().is_empty() {
            message = "The training dataset is missing".to_string();
        } else {
            let content =
                csv_file::read_csv_file(self.training_file.lock().unwrap().to_string())
                    .map_err(|e| Status::internal(format!("Failed to read CSV file: {}", e)))?;

            let (X_train, y_train, X_test, y_test) = normalize::clean_dataset(content.to_owned());
            let model = training::train_log_reg(&X_train, &y_train);
            accuracy = training::model_accuracy(&model.to_owned(), &X_test, &y_test);
            println!("Accuracy: {}", accuracy);
        }

        let response = file::ResponseAccuracy {
            message: message.into(),
            accuracy: accuracy as f32,
        };

        Ok(Response::new(response))
    }

    async fn launch_prediction(
        &self,
        _request: Request<file::RequestPrediction>,
    ) -> Result<Response<file::ResponsePrediction>, Status> {
        let mut message = String::from("");
        let prediction: Array1<f64> = ArrayBase::zeros(0);
        if self.prediction_file.lock().unwrap().is_empty() {
            message = "The testing dataset is missing".to_string();
        } else if self.coefs_path.lock().unwrap().is_empty() {
            message = "The model coefficients are missing".to_string();
        } else {
            let content = csv_file::read_csv_file(self.prediction_file.lock().unwrap().to_string())
                .map_err(|e| Status::internal(format!("Failed to read CSV file: {}", e)))?;
            let model = csv_file::read_file_to_array1(&self.coefs_path.lock().unwrap())
                .map_err(|e| Status::internal(format!("Failed to read txt file: {}", e)))?;

            let (_X_train, _y_train, X_test, _y_test) =
                normalize::clean_dataset(content.to_owned());

            let _prediction = training::predict(&model.to_owned(), &X_test);
        }

        let response = file::ResponsePrediction {
            message: message.into(),
            prediction: serde_json::to_string(&prediction).unwrap(),
        };

        Ok(Response::new(response))
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
