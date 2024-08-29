-- This file should undo anything in `up.sql`
-- down.sql

DO $$
DECLARE
r RECORD;
BEGIN
-- Loop over all tables
FOR r IN (SELECT tablename FROM pg_tables WHERE schemaname = current_schema()) LOOP
EXECUTE 'DROP TABLE IF EXISTS ' || quote_ident(r.tablename) || ' CASCADE';
END LOOP;
END $$;