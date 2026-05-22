# Invoice & Payment Service API Documentation

## 1. Authentication
Most endpoints require a Bearer token in the `Authorization` header.
- Header: `Authorization: Bearer sk_...`
- The token is obtained via the **Register Business** endpoint.

---

## 2. Businesses
### Register Business
Registers a new business and generates a one-time API key.
- **URL**: `POST /v1/businesses`
- **Body**:
```json
{
  "name": "Acme Corp"
}
```
- **Response**: `200 OK`
```json
{
  "id": "01H...",
  "name": "Acme Corp",
  "api_key": "sk_..."
}
```

---

## 3. Customers
### Create Customer
- **URL**: `POST /v1/customers`
- **Auth**: Required
- **Body**:
```json
{
  "name": "John Doe",
  "email": "john@example.com"
}
```

### List Customers
- **URL**: `GET /v1/customers`
- **Auth**: Required

### Get Customer
- **URL**: `GET /v1/customers/:id`
- **Auth**: Required

---

## 4. Invoices
### Create Invoice
Line items are automatically summed. State defaults to `draft`.
- **URL**: `POST /v1/invoices`
- **Auth**: Required
- **Body**:
```json
{
  "customer_id": "cust_id",
  "line_items": [
    {
      "description": "Widget",
      "quantity": 2,
      "unit_amount_cents": 500
    }
  ],
  "idempotency_key": "optional_unique_key"
}
```

### List Invoices
- **URL**: `GET /v1/invoices`
- **Query Params**: `?state=draft|open|paid|void|uncollectible`
- **Auth**: Required

### Get Invoice (with items)
- **URL**: `GET /v1/invoices/:id`
- **Auth**: Required

### Finalize Invoice
Moves invoice from `draft` to `open`.
- **URL**: `POST /v1/invoices/:id/finalize`
- **Auth**: Required

### Void Invoice
- **URL**: `POST /v1/invoices/:id/void`
- **Auth**: Required

### Mark Uncollectible
- **URL**: `POST /v1/invoices/:id/mark-uncollectible`
- **Auth**: Required

---

## 5. Payments
### Pay Invoice
Requires an `Idempotency-Key` header.
- **URL**: `POST /v1/invoices/:id/pay`
- **Auth**: Required
- **Headers**: `Idempotency-Key: <unique-uuid>`
- **Body**:
```json
{
  "card_token": "tok_success"
}
```
- **PSP Mock Tokens**:
  - `tok_success`: Simulates successful payment.
  - `tok_fail`: Simulates declined card.
  - `tok_timeout`: Simulates PSP timeout (10s).

---

## 6. Webhooks
### Register Webhook
- **URL**: `POST /v1/webhooks`
- **Auth**: Required
- **Body**:
```json
{
  "target_url": "https://webhook.site/..."
}
```

### List Webhooks
- **URL**: `GET /v1/webhooks`
- **Auth**: Required

### Deactivate Webhook
- **URL**: `DELETE /v1/webhooks/:id`
- **Auth**: Required
