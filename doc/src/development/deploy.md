# Deploy

```sh
cargo build --release
```

Copy the final executable (`./target/release/openfairdb`)
to the target directory of your server and make sure it gets
exectuted as a service on startup.

## Docker

### Build the image

Build and tag the Docker image:

```sh
docker build -t openfairdb:latest .
```

The image is created `FROM scratch` and does not provide any user environment or shell.

### Run the container

The executable in the container is controlled by the following environment variables:

- `RUST_LOG`: Log level (trace, debug, info, warn, error)
- `DATABASE_URL`: Database file path

The database file must be placed in a volume outside of the container. For
this purpose the image defines the mountpoint */volume* where an external volume
from the host can be mounted.

The container exposes the port `8080` for publishing to the host.

Example:

```sh
docker run --rm \
    -p 6767:8080 \
    -e RUST_LOG="info" \
    -e DATABASE_URL="/volume/openfairdb.sqlite" \
    -v "/var/openfairdb":/volume:Z \
    openfairdb:latest
```

### Extract the static executable

The resulting Docker image contains a static executable named `entrypoint` that can be extracted
from any container instance (but not directly from the image itself):

```sh
docker cp <container id>:entrypoint openfairdb
```

## Mailing

To be able to send email notifications you need to define
a sender email address. You can do this by setting the
`MAIL_GATEWAY_SENDER_ADDRESS` environment variable.
If you like to use the [mailgun](https://mailgun.com)
service you also need to define the
`MAILGUN_API_KEY` variable with your API key
and the `MAILGUN_DOMAIN` variable with the domain
you are setup for mailgun.

## DB Backups

At the moment the OpenFairDB does not support online backups.
Therefore we use a simple
[script](https://github.com/kartevonmorgen/openfairdb/blob/main/scripts/backup_db.sh)
that copies the DB file once a day.
