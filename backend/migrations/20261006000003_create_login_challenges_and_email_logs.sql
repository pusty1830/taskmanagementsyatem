CREATE TABLE login_challenges (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    -- HMAC-SHA256(OTP_SECRET, challenge_id:code); the plaintext code is never stored here.
    code_hash   TEXT        NOT NULL,
    attempts    INT         NOT NULL DEFAULT 0,
    expires_at  TIMESTAMPTZ NOT NULL,
    consumed_at TIMESTAMPTZ NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_login_challenges_user ON login_challenges(user_id);

-- Development mailbox standing in for real email delivery.
CREATE TABLE email_logs (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    to_email     TEXT        NOT NULL,
    subject      TEXT        NOT NULL,
    body         TEXT        NOT NULL,
    challenge_id UUID        NULL REFERENCES login_challenges(id) ON DELETE SET NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_email_logs_created ON email_logs(created_at DESC);
