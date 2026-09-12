-- Sources and groups used to be two pages, and a single key could be added on
-- either: through Sources it got a `key` source row as well as its profile,
-- through Groups it got only the profile. Now there is one way in and only a
-- subscription URL keeps a source, so the key rows go. Their profiles stay
-- where they are; `ON DELETE SET NULL` detaches them.
DELETE FROM sources WHERE type = 'key';

-- Removing a subscription used to delete only its source, leaving a group that
-- still called itself a subscription with nothing to refresh from. Those are
-- ordinary groups now, and their VPNs ordinary local ones the user can move,
-- rename and delete.
UPDATE profile_groups SET kind = 'custom' WHERE kind = 'subscription' AND source_id IS NULL;
UPDATE profiles SET origin = 'local' WHERE origin = 'imported' AND source_id IS NULL;
