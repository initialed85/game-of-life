#!/bin/bash

set -e

docker build --progress plain --platform=linux/amd64 -t kube-registry:5000/game-of-life:latest -f ./Dockerfile .

docker image push kube-registry:5000/game-of-life:latest
