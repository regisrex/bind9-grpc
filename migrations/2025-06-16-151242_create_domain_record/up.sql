CREATE TABLE domain_records (
    id SERIAL PRIMARY KEY, -- Maps to Record.id, assuming auto-incrementing integer ID
                           -- If Record.id is a UUID string, use UUID PRIMARY KEY or TEXT PRIMARY KEY
    
    host TEXT NOT NULL, -- Maps to Record.host
    ttl BIGINT,                -- Maps to Record.ttl (NULL for Option<u64>)
    class TEXT NOT NULL,       -- Maps to Record.class
    
    -- Maps to Record.record_type, storing the enum variant name as text
    record_type TEXT NOT NULL, 
    
    mx_preference BIGINT,      -- Maps to Record.mx_preference (NULL for Option<u64>)
    value TEXT NOT NULL,    -- Maps to Record.value
    
    -- Fields for SOAProperties (all nullable, as SOAProperties is an Option)
    soa_host_mail TEXT,          -- Maps to SOAProperties.host_mail
    soa_serial_number BIGINT,    -- Maps to SOAProperties.serial_number
    soa_refresh_time BIGINT,     -- Maps to SOAProperties.refresh_time
    soa_retry_time BIGINT,       -- Maps to SOAProperties.retry_time
    soa_expire_time BIGINT,      -- Maps to SOAProperties.expire_time
    soa_negative_cache_ttl BIGINT, -- Maps to SOAProperties.negative_cache_ttl
    is_deleted boolean  DEFAULT false

);

-- Optional: Add a CHECK constraint to ensure valid record types
ALTER TABLE domain_records
ADD CONSTRAINT chk_record_type CHECK (
    record_type IN ('A', 'NS', 'CNAME', 'SOA', 'PTR', 'MX', 'TXT', 'AAAA', 'SRV')
);

-- Optional: If you have a 'domains' table, you might want to link records to it
-- This assumes a 'domains' table exists with a 'domain' TEXT column or 'id' SERIAL/INT column
ALTER TABLE domain_records
ADD COLUMN domain_id INTEGER REFERENCES domains(id) ON DELETE CASCADE;
-- OR
-- ALTER TABLE domain_records
-- ADD CONSTRAINT fk_domain FOREIGN KEY (domain_name) REFERENCES domains(domain) ON UPDATE CASCADE;