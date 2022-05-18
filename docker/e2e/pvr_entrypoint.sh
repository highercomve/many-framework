#!/bin/bash
set -e

cp -n -R /tendermint_init/* $TMHOME || true

exec /usr/local/bin/docker-entrypoint.sh "$@"
