#!/usr/bin/env sh

mkdir -p stage
cp ../../centos-packages ./stage/
docker build -t eozd/libreguitar-cross:x86_64-unknown-linux-gnu .
