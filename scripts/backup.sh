#!/bin/sh
# Neo4j community edition backup script
# 2015 by Markus Kohlhase <mail@markus-kohlhase.de>

BACKUP_SRC=/var/lib/neo4j/data/graph.db # which folder to backup
BACKUP_DEST=/var/lib/neo4j/backup       # where to store the backups
SERVICE_NAME=neo

echo "Shutting down neo4j server to perform a backup..."
systemctl stop $SERVICE_NAME
sleep 1
while pgrep $SERVICE_NAME; do
  sleep 1
done

TIMESTAMP=`date +%Y-%m-%d-%H%M%S`
FILENAME=graph.db-$TIMESTAMP.tar.gz

echo "Creating neo4j DB backup"
tar -czf $BACKUP_DEST/$FILENAME $BACKUP_SRC
echo "Starting neo4j server..."

systemctl start $SERVICE_NAME
sleep 1
while ! pgrep $SERVICE_NAME; do
  sleep 1
done
