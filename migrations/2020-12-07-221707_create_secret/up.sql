CREATE TABLE Secrets (
    id SERIAL PRIMARY KEY,
    document TEXT NOT NULL UNIQUE
)