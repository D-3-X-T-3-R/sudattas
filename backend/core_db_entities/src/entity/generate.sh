#!/usr/bin/env sh


set -e
set -x

rm -f *.rs
~/.cargo/bin/sea-orm-cli generate entity -u mysql://root:12345678@13.233.125.216:3306/SUDATTAS --with-serde=both --date-time-crate chrono --max-connections=1000 
# ~/.cargo/bin/sea-orm-cli generate entity -u mysql://root:12345678@localhost:3306/SUDATTAS --with-serde=both --date-time-crate chrono --max-connections=1000 