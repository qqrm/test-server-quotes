# Test Server Quotes

`test-server-quotes` is an Actix Web-based application serving as an example for demonstrating user authentication, state management, and quote retrieval.

## Features

- **Timestamp Retrieval:** Retrieve the current UNIX timestamp for authentication purposes.
- **User Authentication:** Users can authenticate by hashing the retrieved timestamp.
- **Quote Retrieval:** Once authenticated, users can fetch random quotes.
- **Logout:** Authenticated users can end their session.

## Dependencies

- Rust 2021 Edition
- Actix Web
- MD5 for hashing
- Futures for asynchronous programming
- Rand for random number and choice generation

## Setup & Running

1. **Clone the Repository:**
   ```bash
   git clone https://github.com/your-username/test-server-quotes.git
   ```

2. **Navigate to the Directory:**
   ```bash
   cd test-server-quotes
   ```

3. **Build & Run:**
   ```bash
   cargo run
   ```

The server will start at `127.0.0.1:9999`.

## Endpoints

1. **Get Time:** POST `/time`
2. **Authenticate:** POST `/auth`
3. **Get Quote:** POST `/quote`
4. **Logout:** POST `/logout`
