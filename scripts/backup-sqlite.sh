#!/bin/sh
# SQLite backup script
# 2017 by Markus Kohlhase <markus.kohlhase@slowtec.de>

BACKUP_SRC=/var/db/openfairdb/bin/db.sqlite # which file to backup
BACKUP_DEST=/var/db/openfairdb/bin/backup   # where to store the backups
SERVICE_NAME=ofdb
TIMEOUT=5

CNT=0
echo "Shutting down OpenFairDB server to perform a backup..."
systemctl stop $SERVICE_NAME
sleep 1
while [ $CNT -lt $TIMEOUT ] && pgrep $SERVICE_NAME; do
  echo $CNT
  sleep 1
  CNT=`expr $CNT + 1`
done

TIMESTAMP=`date +%Y-%m-%d-%H%M%S`
FILENAME=db.sqlite-$TIMESTAMP.tar.gz

echo "Creating SQLite DB backup"
tar -czf $BACKUP_DEST/$FILENAME $BACKUP_SRC
echo "Starting OpenFairDB server..."

systemctl start $SERVICE_NAME
sleep 1
while ! pgrep $SERVICE_NAME; do
  sleep 1
done
