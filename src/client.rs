use file::file_client::FileClient;
use file::FileFinished;
use file::FileRequest;
use file::FileTransfer;
use file::RequestPrediction;
use file::RequestTraining;

use http::uri::Uri;

use prost::encoding::bool;
use tonic::transport::{Certificate, Channel, ClientTlsConfig, Identity};

use ring::digest::{Context, SHA256};

use hmac::{Hmac, Mac};
use sha2::Sha256;

mod csv_file;

pub mod file {
    tonic::include_proto!("file");
}

// Create alias for HMAC-SHA256
type HmacSha256 = Hmac<Sha256>;

// Function to upload file to server
async fn upload_file(
    client: &mut FileClient<Channel>,
    file_path: &str,
    train: u32,
    hmac: &mut HmacSha256,
) -> Result<(), Box<dyn std::error::Error>> {
    let filepath = std::path::Path::new(&file_path);
    // Keep only the filename
    let filename_ = filepath.file_name().unwrap().to_str().unwrap().to_string();
    println!("Filename: {}", filename_);

    let mut serialized_data: Vec<u8> = Vec::new();

    // If the train variable is equal to 1, then the file is a training file
    // and the server will save it in the training folder
    if train == 1 || train == 2 {
        if !filename_.ends_with(".csv") {
            println!("Training file must be a .csv file!");
            return Ok(());
        }
        // Read the file
        let content = csv_file::read_csv_file(file_path.to_string())?;
        // Serialize the records using bincode
        serialized_data = bincode::serialize(&content)?;
    }
    if train == 3 {
        if !filename_.ends_with(".txt") {
            println!("Prediction file must be a .txt file!");
            return Ok(());
        }
        // Read the file
        let content = csv_file::read_file_to_array1(&file_path.to_string())?;
        // Serialize the records using bincode
        serialized_data = bincode::serialize(&content)?;
    }

    // Split the file into chunks of data to send
    let chunks = serialized_data
        .chunks(1024)
        .map(|chunk| chunk.to_vec())
        .collect::<Vec<_>>();

    let request = tonic::Request::new(FileRequest {
        filename: filename_.to_string(),
        train: train == 1,
        coefs: filename_.ends_with(".txt"),
    });

    let mut response = client.priming_send(request).await?;

    match response.into_inner().message.as_str() {
        "OK" => (),
        _ => {
            println!("Error during the priming of the transfer");
            return Ok(());
        }
    }

    println!("Uploading {} to the server...", filename_);

    for chunk in chunks {
        // Calculate the SHA-256 hash of the chunk
        let mut context = Context::new(&SHA256);
        context.update(&chunk);
        let chunk_hash = context.finish();

        let request = tonic::Request::new(FileTransfer {
            filename: filename_.clone(),
            content: chunk,
            hash: chunk_hash.as_ref().to_vec(), // Include the hash in the FileTransfer message
        });

        response = client.send_file(request).await?;

        match response.into_inner().message.as_str() {
            "OK" => (),
            _ => {
                println!("File upload failed!");
                return Ok(());
            }
        }
    }

    // Compute the HMAC hash of the entire serialized file
    hmac.update(&serialized_data);
    let hmac_hash = hmac.clone().finalize().into_bytes().to_vec();

    let newrequest = tonic::Request::new(FileFinished {
        filename: filename_.to_string(),
        hmac_hash: hmac_hash,
    });

    println!("Finishing the transfer...");

    client.finish_transfer(newrequest).await?;

    println!("File uploaded successfully!");
    Ok(())
}

async fn start_prediction(
    client: &mut FileClient<Channel>,
) -> Result<(), Box<dyn std::error::Error>> {
    let request = tonic::Request::new(RequestPrediction { predict: true });

    let response = client.launch_prediction(request).await?;

    // Print the response
    println!("RESPONSE={:?}", response);

    Ok(())
}

async fn start_training(
    client: &mut FileClient<Channel>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Launching training...");

    let request = tonic::Request::new(RequestTraining { train: true });

    let response = client.launch_training(request).await?;

    // Print the response
    println!("RESPONSE={:?}", response);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get the args passed to the program
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 3 {
        println!("Usage: {} <host> <port>", args[0]);
        return Ok(());
    }

    let host = &args[1];
    let port = &args[2];

    let server_root_ca_cert = std::fs::read_to_string("ca.crt")?;
    let server_root_ca_cert = Certificate::from_pem(server_root_ca_cert);
    let client_cert = std::fs::read_to_string("client.crt")?;
    let client_key = std::fs::read_to_string("client.key")?;
    let client_identity = Identity::from_pem(client_cert, client_key);

    let tls = ClientTlsConfig::new()
        .domain_name("localhost")
        .ca_certificate(server_root_ca_cert)
        .identity(client_identity);

    let uri: Uri = format!("http://{}:{}", host, port).parse()?;
    let channel = Channel::builder(uri).tls_config(tls)?.connect().await?;

    let mut client = FileClient::new(channel);

    let mut choice = String::new();
    // Put the number 0 in choice to enter the loop
    choice.push('0');

    loop {
        // Ask the user what does he want to do with the server
        println!("What do you want to do?");
        println!("1. Upload a file for training");
        println!("2. Upload a file for prediction");
        println!("3. Upload a trained model");
        println!("4. Launch the training");
        println!("5. Launch the prediction");
        println!("6. Exit");

        std::io::stdin().read_line(&mut choice)?;

        match choice.trim().parse::<u32>() {
            Ok(1 | 2 | 3) => {
                let mut hmac =
                    HmacSha256::new_from_slice(b"secret").expect("HMAC can take key of any size");
                // Ask the user for the name of the file to upload
                println!("Enter the path of the file to upload:");
                let mut filepath = String::new();
                std::io::stdin().read_line(&mut filepath)?;
                // Upload a file for training
                upload_file(
                    &mut client,
                    filepath.trim(),
                    choice.trim().parse::<u32>().unwrap(),
                    &mut hmac,
                )
                .await?;
            }
            Ok(4) => {
                // Launch the training
                start_training(&mut client).await?;
            }
            Ok(5) => {
                // Launch the prediction
                start_prediction(&mut client).await?;
            }
            Ok(6) => {
                // Exit the program
                println!("Exiting...");
                return Ok(());
            }
            _ => {
                println!("Invalid choice!");
            }
        }
        // clear the choice variable
        choice.clear();
    }
}
