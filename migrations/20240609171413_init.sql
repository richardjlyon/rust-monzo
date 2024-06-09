-- Base app schema

CREATE TABLE accounts (
    id TEXT PRIMARY KEY NOT NULL ,
    closed BOOLEAN NOT NULL,
    created DATETIME NOT NULL,
    description TEXT NOT NULL ,
    owner_type TEXT  NOT NULL,
    account_number TEXT NOT NULL,
    sort_code TEXT NOT NULL
);

CREATE TABLE pots (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    balance INTEGER NOT NULL,
    currency TEXT NOT NULL,
    deleted BOOLEAN NOT NULL
);

CREATE TABLE transactions (
    id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL,
    merchant_id TEXT,
    amount INTEGER NOT NULL,
    currency TEXT NOT NULL,
    local_amount INTEGER NOT NULL,
    local_currency TEXT NOT NULL,
    created DATETIME NOT NULL,
    description TEXT,
    notes TEXT,
    settled DATETIME,
    updated DATETIME,
    category TEXT,

    FOREIGN KEY(account_id) REFERENCES accounts(id),
    FOREIGN KEY(merchant_id) REFERENCES merchants(id)
);

CREATE TABLE merchants (
    id TEXT PRIMARY KEY,
    name TEXT,
    category TEXT
);