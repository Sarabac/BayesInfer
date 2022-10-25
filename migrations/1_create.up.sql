-- Add up migration script here
CREATE TABLE IF NOT EXISTS hypo (
    id SERIAL PRIMARY KEY,
    json_serial VARCHAR NOT NULL,
    UNIQUE(json_serial)
);
CREATE TABLE IF NOT EXISTS record (
    id SERIAL PRIMARY KEY,
    json_serial VARCHAR NOT NULL,
    UNIQUE(json_serial)
);
CREATE TABLE IF NOT EXISTS model (
    id SERIAL PRIMARY KEY,
    hypo_id integer REFERENCES hypo(id),
    record_id integer REFERENCES record(id),
    proba double precision,
    UNIQUE(hypo_id, record_id)
);
CREATE TABLE IF NOT EXISTS likelihood (
    id SERIAL PRIMARY KEY,
    hypo_id integer REFERENCES hypo(id),
    iter integer,
    proba double precision,
    UNIQUE(hypo_id, iter)
);