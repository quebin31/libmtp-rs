#!/bin/bash


LIBMTP_SRC="$1"
OUTPUT="$2"

pushd "${LIBMTP_SRC}"
./autogen.sh
popd

bindgen wrapper.h \
    -o "${OUTPUT:-src/bindings.rs}" \
    --whitelist-function '^LIBMTP_.*' \
    --whitelist-var '^LIBMTP_.*' \
    --whitelist-type '^LIBMTP_.*' \
    --blacklist-type '^__.*' \
    --blacklist-type 'time_t' \
    --blacklist-type 'timeval' \
    --raw-line 'pub type time_t = libc::time_t;' \
    --raw-line 'pub type timeval = libc::timeval;' \
    --whitelist-var '^PTP_ST_.*' \
    --whitelist-var '^PTP_FST_.*' \
    --whitelist-var '^PTP_AC_.*' \
    -- -I "./libmtp-1.1.17/src"