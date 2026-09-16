-- `last_refresh` held English prose written by the backend ("Updated just
-- now"). It never aged, so a subscription refreshed a week ago still claimed
-- to be current, and the frontend could only translate that one exact phrase -
-- anything else reached the screen in English. The column now holds the
-- refresh time as unix milliseconds, with the empty string for "never".
--
-- Existing prose cannot be turned back into a time, so it is cleared. Those
-- rows read as never refreshed until their next refresh, which is the only
-- honest answer available.
UPDATE sources SET last_refresh = '';
