-- Refund worker previously retried a permanently-failing gateway refund (e.g. bad payment_id)
-- forever with no terminal state. Add an attempt counter so the worker can give up after a
-- configurable number of retries and mark the attempt 'needs_review' for manual handling.

SET @refund_attempts_attempt_count_exists := (
    SELECT COUNT(*)
    FROM information_schema.columns
    WHERE table_schema = DATABASE()
      AND table_name = 'RefundAttempts'
      AND column_name = 'attempt_count'
);
SET @refund_attempts_attempt_count_sql := IF(
    @refund_attempts_attempt_count_exists = 0,
    'ALTER TABLE RefundAttempts ADD COLUMN attempt_count INT NOT NULL DEFAULT 0',
    'SELECT 1'
);
PREPARE refund_attempts_attempt_count_stmt FROM @refund_attempts_attempt_count_sql;
EXECUTE refund_attempts_attempt_count_stmt;
DEALLOCATE PREPARE refund_attempts_attempt_count_stmt;
