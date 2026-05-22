-- Seed a business
INSERT INTO businesses (id, name, api_key_hash, api_key_prefix)
VALUES ('01AN4Z07BY79KA1307SR9X4MV3', 'Test Business', '=19=4096,t=3,p=1', 'sk_test_');

-- Seed a customer
INSERT INTO customers (id, business_id, name, email)
VALUES ('01AN4Z07BY79KA1307SR9X4MV4', '01AN4Z07BY79KA1307SR9X4MV3', 'John Doe', 'john@example.com');
