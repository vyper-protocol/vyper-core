#!/bin/bash

PROGRAM_NAME=$1
CLUSTER=$2
VERSION=$3

echo git tag -a ${PROGRAM_NAME}_${CLUSTER}_v${VERSION} -m \"${PROGRAM_NAME} v${VERSION} deployment on ${CLUSTER}\"
echo git push origin --tags