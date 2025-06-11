-- Add migration script here
CREATE TABLE solar_data (
    id SERIAL PRIMARY KEY,
    current_kwh DOUBLE PRECISION NOT NULL,
    raw_data JSONB NOT NULL,
    created_at TIMESTAMP WITHOUT TIME ZONE DEFAULT now() NOT NULL
);

CREATE TABLE cached_token (
    id SERIAL PRIMARY KEY,
    login_data JSONB NOT NULL,
    created_at TIMESTAMP WITHOUT TIME ZONE DEFAULT now() NOT NULL
);
