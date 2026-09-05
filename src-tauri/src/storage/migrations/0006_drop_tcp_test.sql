-- The TCP ping is gone. It measured the round trip to the proxy server rather
-- than through it, which told a user nothing they could act on, and it left
-- two columns showing a number the URL test already reports better.
ALTER TABLE profiles DROP COLUMN tcp_tone;
ALTER TABLE profiles DROP COLUMN tcp_value;
