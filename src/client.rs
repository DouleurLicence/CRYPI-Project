use file::file_client::FileClient;
use file::FileFinished;
use file::FileRequest;
use file::FileTransfer;

mod csv_file;

pub mod file {
    tonic::include_proto!("file");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get the args passed to the program
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 4 {
        println!("Usage: {} <host> <port> <file>", args[0]);
        return Ok(());
    }

    let addr = "http://".to_owned() + &args[1] + ":" + &args[2];

    let mut client = FileClient::connect(addr).await?;

    let filename_ = args[3].split("/").last().unwrap().to_string();

    // Check if the file is a CSV file
    if !filename_.ends_with(".csv") {
        println!("The file must be a CSV file!");
        return Ok(());
    }

    // Read the file
    let content = csv_file::read_csv_file(args[3].clone())?;
    // Serialize the records using bincode
    let serialized_data = bincode::serialize(&content)?;
    // Split the file into chunks of data to send
    let chunks = serialized_data
        .chunks(1024)
        .map(|chunk| chunk.to_vec())
        .collect::<Vec<_>>();

    // Send a first request with the filename to ammorce the transfer

    let request = tonic::Request::new(FileRequest {
        filename: filename_.to_string(),
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
        let request = tonic::Request::new(FileTransfer {
            filename: filename_.clone(),
            content: chunk,
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

    let newrequest = tonic::Request::new(FileFinished {
        filename: filename_.to_string(),
    });

    client.finish_transfer(newrequest).await?;

    println!("File uploaded successfully!");

    Ok(())
}
