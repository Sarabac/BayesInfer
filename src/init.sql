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
    proba double precision
);
CREATE TABLE IF NOT EXISTS likelihood (
    id SERIAL PRIMARY KEY,
    hypo_id integer REFERENCES hypo(id),
    iter integer,
    proba double precision
);
CREATE OR REPLACE VIEW bayes_prior AS
SELECT m.hypo_id AS hypo_id,
    m.record_id AS record_id,
    l.iter AS iter,
    m.proba * l.proba AS proba
FROM model m
    INNER JOIN likelihood l ON m.hypo_id = l.hypo_id;
CREATE OR REPLACE VIEW transfert AS
SELECT m.hypo_id AS hypo_id,
    m.record_id AS record_id,
    l.iter AS iter,
    m.proba / l.proba AS proba
FROM bayes_prior m
    INNER JOIN (
        SELECT record_id,
            iter,
            SUM(proba) AS proba
        FROM bayes_prior
        GROUP BY record_id,
            iter
    ) l ON m.record_id = l.record_id;