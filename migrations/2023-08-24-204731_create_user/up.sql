CREATE TYPE IF NOT EXISTS MULTI_FACTOR_METHOD AS ENUM ('third_party_app', 'email');

CREATE TABLE IF NOT EXISTS Users (
    id SERIAL PRIMARY KEY,
    uuid VARCHAR(128) NOT NULL UNIQUE,
    name VARCHAR(64) NOT NULL UNIQUE,
    email VARCHAR(64) NOT NULL UNIQUE,
    actual_email VARCHAR(64) NOT NULL UNIQUE,
    password VARCHAR(128) NOT NULL,
    multi_factor_method MULTI_FACTOR_METHOD,
);