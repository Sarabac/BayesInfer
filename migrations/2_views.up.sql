-- Add up migration script here
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