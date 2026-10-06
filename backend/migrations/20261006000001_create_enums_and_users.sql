CREATE TYPE user_role     AS ENUM ('admin', 'staff');
CREATE TYPE task_status   AS ENUM ('todo', 'in_progress', 'done');
CREATE TYPE task_priority AS ENUM ('low', 'medium', 'high');

CREATE TABLE users (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    full_name       TEXT        NOT NULL CHECK (length(full_name) BETWEEN 1 AND 120),
    email           TEXT        NOT NULL UNIQUE CHECK (email = lower(email)),
    hashed_password TEXT        NOT NULL,
    role            user_role   NOT NULL DEFAULT 'staff',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
