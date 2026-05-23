# Invoice & Payment Service

A  invoice management and payment processing system built with Rust, Axum, and SQLx.

## Core Features
- **DDD Architecture**: Strict separation of concerns (DAL, Handlers, Services, Routes).
- **Secure Auth**: Argon2id hashed API keys.
- **Transactional Integrity**: Atomic invoice creation and payment processing.
- **Asynchronous Webhooks**: Decoupled delivery with HMAC signing and exponential backoff.
- **Automated Jobs**: Background overdue invoice handling and webhook retry loops.

## Getting Started
### Docker (Recommended)
```bash
docker-compose up --build
```

### Local Development
1. Ensure PostgreSQL is running.
2. Configure `.env` with `DATABASE_URL`.
3. `cargo run`

## Testing
The system includes mandatory integration tests for **Concurrency**, **Idempotency**, and **PSP Failure handling**.
```bash
cargo test
```

## Documentation
- [DESIGN.md](./DESIGN.md): Technical architecture and security details.
- [WALKTHROUGH.md](./WALKTHROUGH.md): Step-by-step usage guide.
- [API.md](./docs/API.md): Full endpoint reference.
- [Postman Collection](./docs/postman_collection.json): Importable collection for testing.

## Video walkthrough
https://drive.google.com/file/d/1BbTZYmp7aeZyOcCxAabV-s5ZJxl64ao9/view?usp=sharing
