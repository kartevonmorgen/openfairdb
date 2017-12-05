#!/bin/sh
# 2017 by Markus Kohlhase <markus.kohlhase@slowtec.de>

# This script requires:
#  - curl
#  - jq

EXPORT_DIR='csv-export'
QUERY_DIR='cypher_export_queries'
C_TYPE='content-type:application/json'
URL='http://localhost:7474/db/data/transaction/commit'
HEADER='accept:application/json'
QUERIES=(
  "entries"
  "old_entries"
  "categories"
  "comments"
  "tags"
  "users"
  "ratings"
  "bbox_subscriptions"
  "entry_category_relations"
  "entry_tag_relations"
)

mkdir -p  $EXPORT_DIR

FetchData(){
  curl -H $HEADER -H $C_TYPE \
    -d @$QUERY_DIR/$1.json $URL \
    | jq -r '(.results[0]) | .data[].row | @csv' \
    > $EXPORT_DIR/$1.csv
}

for name in ${QUERIES[@]}; do
  echo "Processing '$name'"
  FetchData $name
done
