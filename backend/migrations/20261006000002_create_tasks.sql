CREATE TABLE tasks (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title          TEXT          NOT NULL CHECK (length(title) BETWEEN 1 AND 200),
    description    TEXT          NOT NULL DEFAULT '',
    status         task_status   NOT NULL DEFAULT 'todo',
    priority       task_priority NOT NULL DEFAULT 'medium',
    created_by_id  UUID          NOT NULL REFERENCES users(id),
    assigned_to_id UUID          NULL     REFERENCES users(id),
    created_at     TIMESTAMPTZ   NOT NULL DEFAULT now(),
    updated_at     TIMESTAMPTZ   NOT NULL DEFAULT now()
);

CREATE INDEX idx_tasks_assigned_to ON tasks(assigned_to_id);
