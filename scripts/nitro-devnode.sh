#!/bin/bash

MYDIR=$(realpath "$(dirname "$0")")
cd "$MYDIR" || exit

HAS_INIT=false

while [[ $# -gt 0 ]]
do
  case "$1" in
    -i|--init)
      HAS_INIT=true
      shift
      ;;
    -q|--quit)
      NITRO_CONTAINERS=$(docker container ls -q --filter name=nitro)

      if [ -z "$NITRO_CONTAINERS" ]; then
          echo "No nitro node containers running"
      else
          docker container stop $NITRO_CONTAINERS || exit
      fi

      exit 0
      ;;
    *)
      echo "OPTIONS:"
      echo "-i|--init:         clone repo and init nitro node"
      echo "-q|--quit:         shutdown nitro node docker containers"
      echo "-s|--stylus:       setup nitro node with Stylus dev dependencies"
      exit 0
      ;;
  esac
done

DEV_NODE_DIR="$MYDIR/../nitro-devnode"
if [ ! -d "$DEV_NODE_DIR" ]; then
  HAS_INIT=true
fi

if $HAS_INIT
then
  cd "$MYDIR" || exit
  cd ..

  git clone --recurse-submodules https://github.com/OffchainLabs/nitro-devnode.git
  cd ./nitro-devnode || exit
  git pull origin release --recurse-submodules
  git checkout 6cad5efa9c1d57ed9479c66701532f62e14d06c9 || exit

fi


cd "$DEV_NODE_DIR" || exit
./run-dev-node.sh
