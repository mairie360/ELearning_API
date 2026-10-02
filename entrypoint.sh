#!/bin/bash
set -e

# Same directory as the WORKDIR of development.Dockerfile
cd /usr/src/elearning

# Lancer cargo watch
exec cargo watch --poll -w src -i target -x run
