-- xFrame5 Code Generator - Database Initialization Script
--
-- This script runs automatically when the PostgreSQL container starts.
-- It creates the required extensions and initial data.

-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Create default admin user if not exists
-- Password: admin123 (should be changed in production)
INSERT INTO users (pid, email, password, name, api_key, email_verified_at, created_at, updated_at)
SELECT
    uuid_generate_v4(),
    'admin@example.com',
    '$argon2id$v=19$m=19456,t=2,p=1$VE0e3g7DalWHgDwou3nuRA$uC6TER156UQpk0lNQ5+jHM0l5poVjPA1he/Tyn9J4Zw',
    'Admin',
    'admin-api-key-change-me',
    NOW(),
    NOW(),
    NOW()
WHERE NOT EXISTS (
    SELECT 1 FROM users WHERE email = 'admin@example.com'
);

-- Insert default prompt template for xFrame5 list screens
INSERT INTO prompt_templates (name, product, screen_type, system_prompt, user_prompt_template, version, is_active, created_at, updated_at)
SELECT
    'xframe5-list-default',
    'xframe5-ui',
    'list',
    'You are an expert xFrame5 frontend code generator. Your task is to generate XML view files and JavaScript event handlers for xFrame5 applications.

RULES:
1. Generate valid xFrame5 XML with proper Dataset and Grid definitions
2. Use proper column bindings between Dataset and Grid
3. Generate JavaScript with standard transaction functions (fn_search, fn_save, fn_delete, fn_add)
4. Follow xFrame5 naming conventions
5. Add TODO comments for any information you need but don''t have
6. NEVER make up API endpoints - use TODO placeholders instead

OUTPUT FORMAT:
Respond with exactly two sections:

--- XML ---
<your XML content here>

--- JS ---
<your JavaScript content here>

Do not include any explanation outside these sections.',
    'Generate an xFrame5 list screen based on the following specification:

{{dsl_description}}

Requirements:
- Screen type: {{screen_type}}
- Screen name: {{screen_name}}
- Datasets: {{datasets}}
- Grid columns: {{grid_columns}}
- Actions: {{actions}}

{{#if notes}}
Additional notes:
{{notes}}
{{/if}}

{{#if company_rules}}
Company-specific rules:
{{company_rules}}
{{/if}}

Generate the XML and JavaScript code following xFrame5 patterns.',
    1,
    true,
    NOW(),
    NOW()
WHERE NOT EXISTS (
    SELECT 1 FROM prompt_templates WHERE name = 'xframe5-list-default' AND product = 'xframe5-ui'
);

-- Insert default prompt template for xFrame5 detail screens
INSERT INTO prompt_templates (name, product, screen_type, system_prompt, user_prompt_template, version, is_active, created_at, updated_at)
SELECT
    'xframe5-detail-default',
    'xframe5-ui',
    'detail',
    'You are an expert xFrame5 frontend code generator. Your task is to generate XML view files and JavaScript event handlers for xFrame5 detail/form screens.

RULES:
1. Generate valid xFrame5 XML with proper Dataset and form control definitions
2. Use proper field bindings with Attribute Map properties
3. Generate JavaScript with standard functions (fn_init, fn_save, fn_delete, fn_validate)
4. Handle form validation properly
5. Add TODO comments for any information you need but don''t have
6. NEVER make up API endpoints - use TODO placeholders instead

OUTPUT FORMAT:
Respond with exactly two sections:

--- XML ---
<your XML content here>

--- JS ---
<your JavaScript content here>

Do not include any explanation outside these sections.',
    'Generate an xFrame5 detail/form screen based on the following specification:

{{dsl_description}}

Requirements:
- Screen type: {{screen_type}}
- Screen name: {{screen_name}}
- Datasets: {{datasets}}
- Form fields: {{form_fields}}
- Actions: {{actions}}

{{#if notes}}
Additional notes:
{{notes}}
{{/if}}

{{#if company_rules}}
Company-specific rules:
{{company_rules}}
{{/if}}

Generate the XML and JavaScript code following xFrame5 patterns.',
    1,
    true,
    NOW(),
    NOW()
WHERE NOT EXISTS (
    SELECT 1 FROM prompt_templates WHERE name = 'xframe5-detail-default' AND product = 'xframe5-ui'
);

-- Log successful initialization
DO $$
BEGIN
    RAISE NOTICE 'Database initialization completed successfully';
END $$;
