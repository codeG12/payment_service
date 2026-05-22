# Invoice & Payment Service API Documentation

## Authentication
All requests (except business registration) require a Bearer token in the `Authorization` header.
`Authorization: Bearer sk_...`

## Businesses
### Register Business
`POST /v1/businesses`
Body: `{ "name": "Business Name" }`
Returns the raw API key once.

## Customers
### Create Customer
`POST /v1/customers`
Body: `{ "name": "John Doe", "email": "john@example.com" }`

### List Customers
`GET /v1/customers`

### Get Customer
`GET /v1/customers/:id`

