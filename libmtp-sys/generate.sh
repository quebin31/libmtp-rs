#!/bin/bash

bindgen libmtp.h \
    -o src/bindings.rs \
    --whitelist-function '^LIBMTP_.*' \
    --whitelist-var '^LIBMTP_.*' \
    --whitelist-type '^LIBMTP_.*' \
    --blacklist-type '^__.*' \
    --blacklist-type 'time_t' \
    --blacklist-type 'timeval' \
    --raw-line 'pub type time_t = libc::time_t;' \
    --raw-line 'pub type timeval = libc::timeval;' \