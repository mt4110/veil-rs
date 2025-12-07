CREATE TABLE rules (
    id TEXT PRIMARY KEY,
    pattern TEXT NOT NULL,
    description TEXT,
    severity TEXT NOT NULL DEFAULT 'Medium',
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Seed some default rules
INSERT INTO rules (id, pattern, description, severity) VALUES 
('server_rule_1', 'REMOTE_SECRET_\d+', 'Demo Remote Rule', 'High'),
('legacy_auth', 'auth_token=[a-zA-Z0-9]+', 'Legacy Auth Pattern', 'Medium');
