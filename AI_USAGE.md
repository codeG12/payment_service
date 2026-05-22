# AI Usage Log

- Initialized project with Axum, SQLx, and PostgreSQL.
- Implemented Domain-Driven Design architecture with Business, Customers, Invoices, Billing, Webhooks, and Security domains.
- Created shared infrastructure for error handling and application state.
- Implemented Customers domain (Models, DAL, Services, Handlers, and Routes).
- Added initial database migration for customers table.
- Configured tracing for logging.

- Created a rough draft of Database model and used claude sonnet 4.6 to review the data-model to know the potential improvements it provided comments like
```
invoice: rename money column to add currency CHAR(3),  keep line_items as JSONB or extract to a line_items table (chose to keep it as line_items table since keeping it as jsonb includes pain of handling the data-integrity and  minimize redundancy and we can use query to perform calculation rather than taking and processing it in code)

payment_attempt (rename from payment): currency, error_message, psp_reference_id, card_token

session: remove entirely; API key auth is stateless — no sessions needed(I thought session might be neeeded later wit reading api-based authentication is stateless)

webhook_endpoint (rename): change secret_token to a random bytes secret for HMAC-SHA256

add webhook_delivery table: id, webhook_endpoint_id, event_type, payload JSONB, state (pending/delivered/failed), attempt_count, next_retry_at, last_error, created_at
(claude added this history table and after a little reading I figured out since we relying on external system state table is indeed must in out scenario)
```

Then I asked claude to generate a implementation plan for database layer creation and seeding the sample data within the postgres,after reviewing the implementation plan and used gemini flash to implement this layer.
