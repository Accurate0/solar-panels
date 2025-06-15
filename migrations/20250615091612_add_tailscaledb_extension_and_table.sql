-- Add migration script here
CREATE EXTENSION IF NOT EXISTS timescaledb;

CREATE TABLE solar_data_tsdb (
    current_kwh DOUBLE PRECISION NOT NULL,
    raw_data JSONB NOT NULL,
    "time" TIMESTAMP WITHOUT TIME ZONE DEFAULT now() NOT NULL
) WITH (
  tsdb.hypertable,
  tsdb.partition_column='time',
  tsdb.orderby='time DESC'
);

