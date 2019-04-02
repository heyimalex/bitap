#!/usr/bin/bash

while true
do
    QUICKCHECK_TESTS=1000 RUST_LOG=quickcheck cargo test qc_ -- --nocapture
    if [[ x$? != x0 ]] ; then
        exit $?
    fi
done
