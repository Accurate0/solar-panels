-- Add migration script here
ALTER TABLE solar_data_tsdb
ADD uv_level DOUBLE PRECISION;

ALTER TABLE solar_data_tsdb
ADD temperature DOUBLE PRECISION;

