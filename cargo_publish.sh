#!/bin/sh

set -euxo pipefail

for i in $(seq 10)
do
    cargo +stable publish && break
    sleep 5
done
