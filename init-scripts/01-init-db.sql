-- Database initialization script for DNS gRPC Service
-- This script runs when the PostgreSQL container starts for the first time

-- Create extensions if they don't exist
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Set timezone
SET timezone = 'UTC';

-- Create additional users/roles if needed
-- CREATE ROLE dns_readonly WITH LOGIN PASSWORD 'readonly_password';
-- GRANT CONNECT ON DATABASE dns_service TO dns_readonly;

-- Log successful initialization
DO $$
BEGIN
    RAISE NOTICE 'DNS Service database initialized successfully at %', NOW();
END $$;
