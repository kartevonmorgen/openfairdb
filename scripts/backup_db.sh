#!/usr/bin/env bash
set -euo pipefail

# 1st argument: SQLite3 database file, e.g. /var/db/ofdb/openfair.db
# 2nd argument: Backup directory, e.g. /var/db/ofdb/backup
# Example: ./backup_db.sh /var/db/ofdb/openfair.db /var/db/ofdb/backup

# Change into directory where this shell script is located
SCRIPT_ROOT=$(cd -P -- "$(dirname -- "$0")" && pwd -P)
cd "${SCRIPT_ROOT}"

DEFAULT_DB_DIR="${SCRIPT_ROOT}"
DEFAULT_DB_FILE=openfair.db

DEFAULT_DB_PATH="${DEFAULT_DB_DIR}/${DEFAULT_DB_FILE}"
DB_PATH="$(realpath -s "${1:-${DEFAULT_DB_PATH}}")"
echo "DB_PATH = ${DB_PATH}"
if [ ! -f "${DB_PATH}" ];
then
    echo "[ERROR] Database file not found: ${DB_PATH}"
    exit 1
fi

DB_FILE="$(basename ${DB_PATH})"
DB_DIR="$(dirname ${DB_PATH})"
echo "DB_DIR = ${DB_DIR}"
if [ ! -d "${DB_DIR}" ];
then
    echo "[ERROR] Invalid or missing database directory: ${DB_DIR}"
    exit 1
fi

DEFAULT_BACKUP_DIR="${DB_DIR}/backup"
BACKUP_DIR="$(realpath -s "${2:-${DEFAULT_BACKUP_DIR}}")"
echo "BACKUP_DIR = ${BACKUP_DIR}"
if [ ! -d "${BACKUP_DIR}" ];
then
    echo "[ERROR] Invalid or missing backup directory: ${BACKUP_DIR}"
    exit 1
fi

let MIN_LOOP_COUNTER=1
let MAX_LOOP_COUNTER=10
let LOOP_COUNTER=MIN_LOOP_COUNTER
while [ ${LOOP_COUNTER} -le ${MAX_LOOP_COUNTER} ];
do
    if [ ${LOOP_COUNTER} -gt ${MIN_LOOP_COUNTER} ];
    then
        echo "Trying again: ${LOOP_COUNTER} of ${MAX_LOOP_COUNTER}"
    fi

    TIMESTAMP=$(date -u +%Y%m%dT%H%M%SZ)

    TMP_DIR="${BACKUP_DIR}/${TIMESTAMP}"
    mkdir -p "${TMP_DIR}"
    if [ ! -d "${TMP_DIR}" ];
    then
        echo "[ERROR] Failed to create temporary directory for backup: ${TMP_DIR}"
        exit 1
    fi

    BACKUP_PATH="$(realpath -sm "${TMP_DIR}/${DB_FILE}")"

    DB_SIZE=$(stat -c%s "${DB_PATH}")
    cp "${DB_PATH}" "${BACKUP_PATH}"
    BACKUP_SIZE=$(stat -c%s "${BACKUP_PATH}")

    if [ "${DB_SIZE}" = "${BACKUP_SIZE}" ];
    then
        ARCHIVE_PATH="$(realpath -sm ${BACKUP_DIR}/${DB_FILE}_${TIMESTAMP}.tar.xz)"
        tar -C "${BACKUP_DIR}" -cJf "${ARCHIVE_PATH}" "${TIMESTAMP}"
        rm -rf "${TMP_DIR}"
        echo "Backup succeeded: ${ARCHIVE_PATH}"
        exit 0
    fi
    rm -rf "${TMP_DIR}"

    echo "[ERROR] Size of copied file differs from original: expected = ${DB_SIZE}, actual = ${BACKUP_SIZE}"
    let LOOP_COUNTER=LOOP_COUNTER+1
done

echo "[ERROR] Backup failed!"
exit 1
