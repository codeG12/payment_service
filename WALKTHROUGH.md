# Project Walkthrough - Invoice & Payment Service

## 1. Setup
- Ensure PostgreSQL is running.
- Set `DATABASE_URL` and `PORT` in `.env`.
- Run `cargo run` (migrations will run automatically).

## 2. Business Registration
- Register a business to get your API key:
  `POST /v1/businesses` with `{ "name": "My Corp" }`
- Save the `api_key` returned. Use it in the `Authorization: Bearer <key>` header for subsequent requests.

## 3. Customer Management
- Create a customer:
  `POST /v1/customers` with `{ "name": "Alice", "email": "alice@example.com" }`

## 4. Invoicing Flow
- **Create**: `POST /v1/invoices` with line items. State will be `draft`.
- **Finalize**: `POST /v1/invoices/:id/finalize`. State moves to `open`.
- **Pay**: `POST /v1/invoices/:id/pay` with a `card_token` (e.g., `tok_success`).
  - Use `tok_fail` to simulate a declined card.
  - Use `tok_timeout` to simulate a PSP timeout.
  - Provide a unique `Idempotency-Key` header.

## 5. Webhooks
- Register a webhook endpoint:
  `POST /v1/webhooks` with `{ "target_url": "https://your-listener.com" }`
- Listen for `invoice.created`, `invoice.paid`, or `invoice.payment_failed` events.
- Verify signatures using the `X-Webhook-Signature` header and your endpoint secret.

## 6. Background Jobs
- The system automatically marks overdue invoices as `uncollectible` every hour.
- Webhook retries occur every 10 seconds for pending deliveries.

