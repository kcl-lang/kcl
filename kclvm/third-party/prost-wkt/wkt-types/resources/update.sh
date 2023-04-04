#!/bin/bash

WORKSPACE=`cargo metadata --format-version 1 | jq -r '.workspace_root '`
RESOURCES="${WORKSPACE}/wkt-types/resources"
VERSION=`cargo metadata --format-version 1 | jq -r '.packages[] | select( .name == "prost" ) | .version '`

REPO="https://raw.githubusercontent.com/tokio-rs/prost/v${VERSION}/prost-types"
LIB_FILE="${REPO}/src/lib.rs"
DATETIME_FILE="${REPO}/src/datetime.rs"

curl --silent ${LIB_FILE} > ${RESOURCES}/lib.rs
curl --silent ${DATETIME_FILE} > ${RESOURCES}/datetime.rs

printf "\n!! Please update ${WORKSPACE}/wkt-types/build.rs to reflect new lib.rs !!\n\n"
