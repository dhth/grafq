#!/usr/bin/env bash

set -e

cypher-shell -u neo4j -p password -f /scripts/load-data.cypher

echo "data loaded successfully"
