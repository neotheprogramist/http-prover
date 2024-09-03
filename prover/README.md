
# Cairo Proving Server

The Cairo Proving Server is the core component responsible for managing and verifying proofs within the Cairo proving system. It handles incoming proof requests, manages authorization, and coordinates the proving tasks.

## Example Usage

Here is an example of running the server with custom settings:

```sh
cairo-prover-server   --host 127.0.0.1   --port 8080   --message-expiration-time 7200   --session-expiration-time 14400   --jwt-secret-key "my_super_secret_key"   --authorized-keys-path /path/to/authorized_keys.json   --authorized-keys "key1,key2,key3"   --num-workes 8   --admin-key "admin_super_secret_key"
```
## Command-Line Options

The server can be configured via command-line arguments or environment variables. The following options are available:

### 1. `--host`

- **Environment Variable:** `HOST`
- **Default:** `0.0.0.0`
- **Example:**

  ```sh
  --host 127.0.0.1
  ```


### 2. `--port`, `-p`

- **Environment Variable:** `PORT`
- **Default:** `3000`
- **Example:**

  ```sh
  --port 8080
  ```


### 3. `--message-expiration-time`, `-m`

- **Environment Variable:** `MESSAGE_EXPIRATION_TIME`
- **Default:** `3600`
- **Example:**

  ```sh
  --message-expiration-time 7200
  ```

### 4. `--session-expiration-time`, `-s`

- **Environment Variable:** `SESSION_EXPIRATION_TIME`
- **Default:** `3600`
- **Example:`

  ```sh
  --session-expiration-time 14400
  ```

### 5. `--jwt-secret-key`, `-k`

- **Environment Variable:** `JWT_SECRET_KEY`
- **Required:** Yes
- **Example:**

  ```sh
  --jwt-secret-key "my_super_secret_key"
  ```


### 6. `--authorized-keys-path`

- **Description:** The path to the JSON file containing authorized public keys.
- **Environment Variable:** `AUTHORIZED_KEYS_PATH`
- **Default:** `authorized_keys.json`
- **Example:**

  ```sh
  --authorized-keys-path /path/to/authorized_keys.json
  ```

### 7. `--authorized-keys`

- **Description:** A comma-separated list of authorized public keys in hex format.
- **Environment Variable:** `AUTHORIZED_KEYS`
- **Type:** `Vec<String>`
- **Example:**

  ```sh
  --authorized-keys key1,key2,key3
  ```

  This provides a list of authorized public keys directly on the command line or via environment variable.

### 8. `--num-workes`

- **Description:** The number of worker threads that the server should use for handling tasks.
- **Environment Variable:** `NUM_WORKES`
- **Default:** `4`
- **Example:**

  ```sh
  --num-workes 8
  ```

### 9. `--admin-key`

- **Description:** The key used for administrative access to the server.
- **Environment Variable:** `ADMIN_KEY`
- **Required:** Yes
- **Example:**

  ```sh
  --admin-key "admin_super_secret_key"
  ```



In this example, the server is configured to:

- Listen on `127.0.0.1` at port `8080`.
- Use a message expiration time of `7200` seconds.
- Use a session expiration time of `14400` seconds.
- Sign JWTs with the provided `jwt-secret-key`.
- Load authorized keys from the specified file path and directly from the command line.
- Use `8` worker threads.
- Use the provided `admin-key` for administrative tasks.

## Environment Variables

All command-line options can also be set via environment variables. This is particularly useful in containerized or cloud environments where passing environment variables is preferred.

## Getting Started

To start the server, ensure you have the correct configuration and run it with the desired options. For more information on how to interact with the server, refer to the [SDK README](../prover-sdk/README.md) and the [Cairo-Prove README](../bin/cairo-prove/README.md).
