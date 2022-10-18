CREATE TABLE IF NOT EXISTS hypo (
    id SERIAL PRIMARY KEY,
    json_serial VARCHAR NOT NULL
);
CREATE TABLE IF NOT EXISTS record (
    id SERIAL PRIMARY KEY,
    json_serial VARCHAR NOT NULL
);
CREATE TABLE IF NOT EXISTS model (
    id SERIAL PRIMARY KEY,
    hypo_id integer REFERENCES hypo(id),
    record_id integer REFERENCES record(id),
    proba FLOAT
);
CREATE TABLE IF NOT EXISTS likelihood (
    id SERIAL PRIMARY KEY,
    hypo_id integer REFERENCES hypo(id),
    iter integer,
    proba double precision
);