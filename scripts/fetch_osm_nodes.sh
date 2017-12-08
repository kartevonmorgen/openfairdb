#!/bin/sh
wget -O nodes.osm 'https://overpass-api.de/api/interpreter?data=[out:json][timeout:1500];(node[~^(organic|diet:vegan|diet:vegetarian|fair_trade|regional|second_hand|charity|ngo|identity)$~^([^nN].*|[nN][^oO].*|[nN][oO].+)$]);out body;'
