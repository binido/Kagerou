-- The resolver is addressed by IP, so only providers whose certificate
-- carries that IP can be listed here - a DoH server named by domain would
-- need the system resolver to find it.
ALTER TABLE settings ADD COLUMN remote_dns TEXT NOT NULL DEFAULT '1.1.1.1'
    CHECK (remote_dns IN ('1.1.1.1', '8.8.8.8', '9.9.9.9', '94.140.14.14'));
