# Design Documentation - Invoice & Payment Service

## 1. API Key Security
- **Generation**: Raw keys are generated as `sk_<base62-32-chars>`.
- **Hashing**: Keys are hashed using **Argon2id** before storage.
- **Prefix**: The first 8 characters are stored in plain text as a prefix to allow fast lookup of potential matches during authentication without hashing every key in the database.
- **Verification**: During authentication, the system fetches all active businesses with the matching prefix and verifies the provided raw key against the stored hash.
- **Revocation**: Setting `is_active = false` immediately invalidates all keys for that business.

## 2. Money Handling
- **Representation**: All monetary values are handled as **integer cents** (64-bit integers).
- **Floats**: Strictly forbidden to avoid precision issues.
- **Computation**: Totals are always computed server-side from line items (`quantity * unit_amount_cents`). Client-supplied totals are ignored to prevent tampering.

## 3. State Machine
| From | To | Trigger |
|---|---|---|
| `draft` | `open` | `POST /finalize` |
| `draft` | `void` | `POST /void` |
| `open` | `void` | `POST /void` |
| `open` | `paid` | Successful Payment |
| `open` | `uncollectible` | Overdue Job |

Transitions are enforced in the service layer. Terminal states (`paid`, `void`, `uncollectible`) prevent any further changes.

## 4. Idempotency
- **Implementation**: Enforced via a `UNIQUE` constraint on `idempotency_key` in the `payment_attempts` table.
- **Behavior**: If a duplicate key is received, the system retrieves and returns the existing attempt record instead of creating a new one.

## 5. Webhook Delivery
- **Decoupling**: Webhook emission is decoupled from the API response using `tokio::spawn`.
- **Signing**: Payloads are signed using **HMAC-SHA256** with a per-endpoint secret.
- **Retries**: Uses exponential backoff (30s, 5m, 30m, 2h) for up to 5 attempts.

## 6. Failure Modes
- **PSP Timeout**: PSP calls have a 10s timeout. If triggered, the attempt is marked as `failed` (reason: `psp_timeout`), and the invoice remains `open`.
- **Atomic Transitions**: Invoice state updates and payment attempt status updates are wrapped in a single database transaction.

