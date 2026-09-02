CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE TYPE transaction_kind AS ENUM ('income', 'expense');

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(), google_subject TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE, name TEXT NOT NULL, avatar_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(), updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE categories (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(), user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL CHECK (char_length(name) BETWEEN 1 AND 60), color TEXT NOT NULL DEFAULT '#6366f1',
    kind transaction_kind NOT NULL, created_at TIMESTAMPTZ NOT NULL DEFAULT now(), UNIQUE (user_id, name, kind)
);
CREATE TABLE transactions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(), user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    category_id UUID REFERENCES categories(id) ON DELETE SET NULL, kind transaction_kind NOT NULL,
    description TEXT NOT NULL CHECK (char_length(description) BETWEEN 1 AND 120),
    amount_cents BIGINT NOT NULL CHECK (amount_cents > 0), transaction_date DATE NOT NULL,
    notes TEXT CHECK (char_length(notes) <= 500), created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX transactions_user_date_idx ON transactions(user_id, transaction_date DESC);
CREATE INDEX transactions_user_category_idx ON transactions(user_id, category_id);

