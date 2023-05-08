FROM ubuntu:latest

RUN apt-get update && \
    apt-get install -y libssl-dev pkg-config make cmake libprotobuf-dev protobuf-compiler
# Use a rust base image
FROM rust:latest

# Set the working directory to /app
WORKDIR /server

# Copy the Rust project files to the container
COPY . .

# Build the Rust project
RUN cargo build --release

RUN ./certificate.sh
# Set the default command to start the Rust client
CMD ["./target/release/server"]

# Expose the default Rust client ports
EXPOSE 28015/tcp
