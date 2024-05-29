#!/bin/bash

MYDIR=$(realpath "$(dirname "$0")")
cd "$MYDIR" || exit

HAS_INIT=false
HAS_DETACH=false

while [[ $# -gt 0 ]]
do
  case "$1" in
    -i|--init)
      HAS_INIT=true
      shift
      ;;
    -d|--detach)
      HAS_DETACH=true
      shift
      ;;
    -down|--shutdown)
      docker container stop "$(docker container ls -q --filter name=nitro-testnode)"
      exit 0
      ;;
    *)
      echo "OPTIONS:"
      echo "-i|--init:         clone repo and init nitro test node"
      echo "-d|--detach:       setup nitro test node in detached mode"
      echo "-down|--shutdown:  shutdown nitro test node docker containers"
      exit 0
      ;;
  esac
done

TEST_NODE_DIR="$MYDIR/nitro-testnode"
if [ ! -d "$TEST_NODE_DIR" ]; then
  HAS_INIT=true
fi

if $HAS_INIT
then
  cd "$MYDIR" || exit
  cd ..
  # clone nitro test node repo
  git clone -b stylus --recurse-submodules https://github.com/OffchainLabs/nitro-testnode.git
  cd ./nitro-testnode || exit
  git checkout 1886f4b89f5c20fd5b0c2cf3d08a009ee73e45ca || exit

  # setup nitro test node
  ./test-node.bash --no-run --init --no-tokenbridge || exit
fi


cd "$TEST_NODE_DIR" || exit
if $HAS_DETACH
then
  ./test-node.bash --detach
else
  ./test-node.bash
fi
