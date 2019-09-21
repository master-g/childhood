#!/usr/bin/env bash

go build
./chr2png --chr ../../example/mario.chr --out dump.png
