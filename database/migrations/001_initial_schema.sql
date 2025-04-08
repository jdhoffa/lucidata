-- Initial schema for the lucidata database
CREATE TABLE cars (
    id SERIAL PRIMARY KEY,
    model VARCHAR(50) NOT NULL,
    mpg FLOAT8,
    cyl INTEGER,
    disp FLOAT8,
    hp INTEGER,
    drat FLOAT8,
    wt FLOAT8,
    qsec FLOAT8,
    vs INTEGER,
    am INTEGER,
    gear INTEGER,
    carb INTEGER
);

CREATE INDEX idx_cars_model ON cars(model);
CREATE INDEX idx_cars_mpg ON cars(mpg);
CREATE INDEX idx_cars_cyl ON cars(cyl);
