#!/bin/bash -x


function testcase() {
    rm -rf devgrounds/
    cargo build
    ./target/debug/devctr init devgrounds
    cd devgrounds/
    ../target/debug/devctr add-repo some_repo
    echo | ../target/debug/devctr add-container run --repo some_repo
    ./run whoami
    ./run git --version
    ./run
    cd ..
}

export CONT_CMD=podman
testcase
export CONT_CMD=docker
testcase