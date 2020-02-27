BEGIN;
ALTER TABLE "records_record" ADD COLUMN "comment" text NULL;
--
-- Add field comment to budget
--
ALTER TABLE "budgets_budget" ADD COLUMN "comment" text NULL;
--
-- Alter field amount on budget
--
--
-- Alter field tags on budget
--
--
-- Create model YearBudget
--
CREATE TABLE "budgets_yearbudget" ("id" serial NOT NULL PRIMARY KEY, "amount_currency" varchar(3) NOT NULL, "amount" numeric(15, 2) NOT NULL, "comment" text NULL, "tags_type" varchar(4) NOT NULL, "tags" text[] NOT NULL, "name" varchar(100) NOT NULL, "year" integer NOT NULL, "user_id" integer NOT NULL);
ALTER TABLE "budgets_yearbudget" ADD CONSTRAINT "budgets_yearbudget_user_id_5a3f1a86_fk_auth_user_id" FOREIGN KEY ("user_id") REFERENCES "auth_user" ("id") DEFERRABLE INITIALLY DEFERRED;
CREATE INDEX "budgets_yearbudget_user_id_5a3f1a86" ON "budgets_yearbudget" ("user_id");
COMMIT;
