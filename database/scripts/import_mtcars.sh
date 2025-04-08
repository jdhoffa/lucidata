#!/bin/bash
set -e

# Convert CSV to SQL inserts
echo "Converting mtcars.csv to SQL inserts..."
psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -c "
CREATE TEMPORARY TABLE temp_cars (
    model VARCHAR(50),
    mpg NUMERIC(5,1),
    cyl INTEGER,
    disp NUMERIC(6,1),
    hp INTEGER,
    drat NUMERIC(4,2),
    wt NUMERIC(5,3),
    qsec NUMERIC(5,2),
    vs INTEGER,
    am INTEGER,
    gear INTEGER,
    carb INTEGER
);

COPY temp_cars FROM '/data/mtcars.csv' WITH (FORMAT csv, HEADER true);

INSERT INTO cars (model, mpg, cyl, disp, hp, drat, wt, qsec, vs, am, gear, carb)
SELECT * FROM temp_cars;

DROP TABLE temp_cars;
"

echo "mtcars data imported successfully!"
