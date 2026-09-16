-- The result of measuring a profile was stored as the English sentence the
-- interface showed ('Not tested', 'Timeout', '42 ms') plus a colour. Sorting
-- by latency meant parsing a number back out of that sentence with a regular
-- expression, translating it meant matching on the English text, and a colour
-- that disagreed with its own value was representable and unnoticeable.
--
-- It is a kind and a number now. The colour is derived from them, so it
-- cannot disagree.
ALTER TABLE profiles ADD COLUMN url_kind TEXT NOT NULL DEFAULT 'notTested'
    CHECK (url_kind IN ('notTested', 'latency', 'timeout', 'noResponse', 'unavailable'));
ALTER TABLE profiles ADD COLUMN url_millis INTEGER;

UPDATE profiles SET
    url_kind = CASE
        WHEN url_value LIKE '% ms' THEN 'latency'
        WHEN url_value = 'Timeout' THEN 'timeout'
        WHEN url_value = 'No response' THEN 'noResponse'
        WHEN url_value = 'Unavailable' THEN 'unavailable'
        ELSE 'notTested'
    END,
    -- SQLite casts the leading digits and stops at the space.
    url_millis = CASE WHEN url_value LIKE '% ms' THEN CAST(url_value AS INTEGER) END;

ALTER TABLE profiles DROP COLUMN url_value;
ALTER TABLE profiles DROP COLUMN url_tone;
