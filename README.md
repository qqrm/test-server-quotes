# Word of Wisdom Server

`test-server-quotes` is a Rust-based application utilizing Actix Web framework to establish a TCP server that provides a series of quotes to authenticated clients. The server incorporates a Proof of Work (PoW) challenge to fend off potential DDoS attacks, enhancing the security protocol before proceeding to quote dissemination.

## Features

- **Timestamp Retrieval:** Clients can fetch the current UNIX timestamp which is essential for the authentication process.
- **User Authentication:** A two-step authentication process whereby clients first retrieve a timestamp, then submit a hash combining the timestamp and their password for verification.
- **Proof of Work Challenge:** A challenge-response protocol that requires clients to solve a PoW challenge before accessing the quote service, acting as a deterrent to DDoS attacks.
- **Quote Retrieval:** Upon successful authentication and PoW verification, clients can request random quotes.
- **Logout:** A feature allowing authenticated clients to end their session securely.
- **Multi-client Testing:** A client-side module for conducting integration tests to validate the server's functionality with concurrent clients.

## Dependencies

- Rust 2021 Edition
- Actix Web for web framework
- MD5 for hashing
- Futures for asynchronous programming
- Rand for random number and choice generation
- Reqwest for HTTP client operations
- Tokio for asynchronous runtime
- Docker for containerization

## Setup & Running

### Local Setup

1. **Clone the Repository:**
   ```bash
   git clone https://github.com/qqrm/test-server-quotes.git
   ```

2. **Navigate to the Directory:**
   ```bash
   cd test-server-quotes
   ```

3. **Build & Run:**
   ```bash
   cargo run --bin server
   cargo run --bin client
   ```

The server will start at `0.0.0.0:80`.

### Docker Setup

1. **Build Docker Images:**
   ```bash
   docker build -t server-image-name -f Dockerfile.server .
   docker build -t client-image-name -f Dockerfile.client .
   ```

2. **Run Docker Containers with Docker Compose:**
   ```bash
   docker-compose up --build
   ```

## PoW Algorithm Explanation

The chosen PoW algorithm is based on the MD5 hashing function. Clients are required to find a nonce such that the hash of the concatenation of the last authentication hash and the nonce starts with a specific number of leading zeros, as defined by the server's current difficulty level. This PoW algorithm provides a simple yet effective way to mitigate DDoS attacks by imposing a computational cost on the client-side, thereby throttling the rate of requests a client can make to the server within a given timeframe.
