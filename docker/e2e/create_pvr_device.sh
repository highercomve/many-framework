#!/usr/bin/bash

set -ex
nodes=$(( $1 - 1 ))
source=${2:-https://pvr.pantahub.com/pantahub-ci/x64_initial_stable}
base=$(realpath $(dirname $0))
dockercompose=$(cat ${base}/genfiles/docker-compose.json)

for node in $(seq 0 "$nodes"); do
  cd $base/genfiles
  rm -rf pvr_node${node} || true
  mkdir pvr_node${node} || true
  cd pvr_node${node}
  pvr init

  parts=$(echo ${dockercompose} | jq -r '.services | keys []' | grep "$node")
  for p in $parts; do
    container=$(echo $p | sed "s/-${node}//")
    image=$(echo ${dockercompose} | jq -r ".services[\"${p}\"].image")
    command=$(echo ${dockercompose} | jq -r ".services[\"${p}\"].command" | sed "s/abci-${node}/0\.0\.0\.0/" | sed "s/ledger-${node}/0\.0\.0\.0/" | sed "s/tendermint-${node}/0\.0\.0\.0/")
    environment=$(echo ${dockercompose} | jq -r ".services[\"${p}\"].environment")    
    sources="local,remote"
    if [ "$container" == "tendermint" ]; then
      sources="remote,local"
    fi
    pvr app add --source=$sources --from=$image $container
    mkdir -p _config/$container/genfiles

    cp $base/genfiles/node${node}/$container* _config/$container/genfiles || true

    if [ "$container" == "tendermint" ]; then
      mkdir -p _config/$container/tendermint_home || true
      cp -r $base/genfiles/node${node}/$container/* _config/$container/tendermint_home
    fi

    tempfile=$(mktemp)
    jq -r --argjson  cmd "$command" '.docker_config.Cmd = $cmd' $container/src.json > $tempfile && cat $tempfile > $container/src.json
    jq -r --argjson  environment "$environment" '.docker_config.Env = $environment' $container/src.json > $tempfile && cat $tempfile > $container/src.json

    pvr app install $container
    pvr add . && pvr commit
    pvr export ../pvr_node${node}.tgz
  done

done
