BEGIN;
ALTER TABLE "records_record" DROP COLUMN "comment";
ALTER TABLE "budgets_budget" DROP COLUMN "comment";
DROP TABLE "budgets_yearbudget";
COMMIT;
