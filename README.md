# Bind9GrcpAPI

A Rust-based DNS management service using gRPC, Diesel (Postgres), and Tonic.


### Prerequisites

- **Rust** (https://rustup.rs/)
- **PostgreSQL** (https://www.postgresql.org/)
- **Diesel CLI** (for migrations)
- **Protobuf Compiler** (`protoc`)
- **Postman** (for gRPC testing)


### 1. Clone the Repository
Just clone this repo

### 2. Create a `.env` File

Create a `.env` file in the project root with the following variables (adjust as needed):

```env
DATABASE_URL=postgres://user:password@localhost/dns_db
DNS_ADDRESS=127.0.0.1:50051
BIND9_PATH=/usr/sbin/named
BIND9_CONFIGFILE=/etc/bind/named.conf
DEFAULT_TTL=3600
HOSTMASTER_EMAIL=hostmaster.example.com  // signifies  hostmaster@example.com  
DNS_HOSTNAME=ns1.example.com
```


### 3. Install Diesel CLI (with Postgres)

```sh
cargo install diesel_cli --no-default-features --features postgres
```


### 4. Install Tonic and Other Dependencies

Dependencies are already specified in `Cargo.toml`. To fetch them:

```sh
cargo fetch
```


### 5. Run Diesel Migrations

Initialize the database and run migrations:

```sh
diesel setup
diesel migration run
# (Optional) To redo the last migration:
diesel migration redo
```


### 6. Build the Project (Generate gRPC Code)

This will also generate the Rust code from your `.proto` files:

```sh
cargo build
```


### 7. Run the Server

```sh
cargo run
```

The gRPC server will start at the address specified in your `.env` (`DNS_ADDRESS`).


### 8. Test gRPC Endpoints with Postman

- Open Postman.
- Switch to the "gRPC" tab.
- Import your `.proto` files from the `proto/` directory.
- Set the server address (e.g., `127.0.0.1:50051`).
- Call the available gRPC methods.


## gRPC API Documentation

This service exposes two main gRPC services for DNS management:

### DomainService

Manages DNS domains.

#### CreateDomain

Creates a new domain with an associated IP address.

**Request: `CreateDomainDto`**
```protobuf
message CreateDomainDto {
  string domain = 1;        // Domain name (e.g., "example.com")
  string ip_address = 2;    // IP address to associate with the domain
}
```

**Response: `RpcDomain`**
```protobuf
message RpcDomain {
  string id = 1;           // Unique domain ID
  string domain = 2;       // Domain name
  int64 default_ttl = 3;   // Default TTL value
}
```

**Example:**
```json
{
  "domain": "example.com",
  "ip_address": "192.168.1.1"
}
```

#### DeleteDomain

Deletes an existing domain by ID.

**Request: `DeleteDomainRequest`**
```protobuf
message DeleteDomainRequest {
  int32 domain_id = 1;  // ID of the domain to delete
}
```

**Response: `DeleteDomainResponse`**
```protobuf
message DeleteDomainResponse {
  bool success = 1;           // Whether deletion was successful
  string message = 2;         // Status message
  repeated string errors = 3; // Any errors encountered
}
```

---

### DomainRecordService

Manages DNS records for domains.

#### CreateDomainRecord

Creates a new DNS record for a domain.

**Request: `CreateDomainRecordDto`**
```protobuf
message CreateDomainRecordDto {
  int64 ttl = 1;             // Time to live in seconds
  string class = 2;          // DNS class (typically "IN" for Internet)
  string host = 3;           // Hostname/subdomain (e.g., "www", "@", "mail")
  string record_type = 4;    // Record type (A, AAAA, CNAME, MX, TXT, NS, PTR, SRV)
  int64 mx_preference = 5;   // MX priority (only for MX records)
  string value = 6;          // Record value (IP, hostname, text, etc.)
  int32 domain_id = 7;       // ID of the parent domain
}
```

**Response: `RpcDomainRecord`**
```protobuf
message RpcDomainRecord {
  int32 id = 1;        // Unique record ID
  string host = 2;     // Hostname
  int64 ttl = 3;       // Time to live
  string type = 4;     // Record type
  int64 priority = 5;  // Priority (for MX records)
  string value = 6;    // Record value
}
```

**Example (A Record):**
```json
{
  "ttl": 3600,
  "class": "IN",
  "host": "www",
  "record_type": "A",
  "value": "192.168.1.1",
  "domain_id": 1
}
```

**Example (MX Record):**
```json
{
  "ttl": 3600,
  "class": "IN",
  "host": "@",
  "record_type": "MX",
  "mx_preference": 10,
  "value": "mail.example.com",
  "domain_id": 1
}
```

#### UpdateDomainRecord

Updates an existing DNS record.

**Request: `UpdateDomainRecordDto`**
```protobuf
message UpdateDomainRecordDto {
  int32 id = 1;       // ID of the record to update
  string host = 2;    // New hostname
  int64 ttl = 3;      // New TTL
  string value = 4;   // New value
  string type = 5;    // New record type
}
```

**Response: `RpcDomainRecord`** (same as CreateDomainRecord response)

#### DeleteDomainRecord

Deletes a DNS record by ID.

**Request: `DeleteDomainRecordDto`**
```protobuf
message DeleteDomainRecordDto {
  int32 id = 1;  // ID of the record to delete
}
```

**Response: `RpcDomainRecord`** (returns the deleted record details)

#### GetRecordById

Retrieves a specific DNS record by ID.

**Request: `GetRecordByIdDto`**
```protobuf
message GetRecordByIdDto {
  int32 id = 1;  // ID of the record to retrieve
}
```

**Response: `RpcDomainRecord`** (same as CreateDomainRecord response)

---

### Supported DNS Record Types

```protobuf
enum DomainRecordType {
  A = 0;      // IPv4 address
  NS = 1;     // Name server
  CNAME = 2;  // Canonical name
  PTR = 3;    // Pointer record
  MX = 4;     // Mail exchange
  TXT = 5;    // Text record
  AAAA = 6;   // IPv6 address
  SRV = 7;    // Service record
}
```

---

### Server Features

- **gRPC Reflection**: Enabled for service discovery
- **Logging Middleware**: Request/response logging via tracing
- **Database**: PostgreSQL with Diesel ORM
- **Transport**: Tonic (gRPC for Rust)

---

### Notes

- Make sure PostgreSQL is running and accessible.
- If you change `.proto` files, re-run `cargo build` to regenerate Rust code.
- For troubleshooting, check `.env` values and database connectivity.
- The server listens on the address specified in `DNS_ADDRESS` environment variable.


### License

MIT
