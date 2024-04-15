# Dockerfile for creating a statically-linked Rust application using Docker's
# multi-stage build feature.
# NOTE: This Dockerfile currently doesn't implement caching

###############################################################################
# Define global ARGs for all stages

# clux/muslrust: /usr/src
# ekidd/rust-musl-builder: /home/rust/src
ARG WORKDIR_ROOT=/usr/src

ARG PROJECT_NAME=openfairdb

ARG BUILD_TARGET=x86_64-unknown-linux-musl

ARG BUILD_MODE=release

ARG BUILD_BIN=${PROJECT_NAME}


###############################################################################
# 1st Build Stage
# The tag of the base image must match the version in the file rust-toolchain!
# Available images can be found at https://hub.docker.com/r/clux/muslrust/tags/
FROM docker.io/clux/muslrust:stable AS build

# Import global ARGs
ARG WORKDIR_ROOT
ARG PROJECT_NAME
ARG BUILD_TARGET
ARG BUILD_MODE
ARG BUILD_BIN

WORKDIR ${WORKDIR_ROOT}

RUN USER=root \
    cargo new --bin ${PROJECT_NAME}
WORKDIR ${WORKDIR_ROOT}/${PROJECT_NAME}

COPY . .

# Test and build the actual project
RUN cargo build --${BUILD_MODE} --target ${BUILD_TARGET} --bin ${BUILD_BIN}

# Switch back to the root directory
#
# NOTE(2019-08-30, uklotzde): Otherwise copying from the build image fails
# during all subsequent builds of the 2nd stage with an unchanged 1st stage
# image. Tested with podman 1.5.x on Fedora 30.
WORKDIR /


###############################################################################
# 2nd Build Stage
FROM scratch

# Import global ARGs
ARG WORKDIR_ROOT
ARG PROJECT_NAME
ARG BUILD_TARGET
ARG BUILD_MODE
ARG BUILD_BIN

ARG DATA_VOLUME="/volume"

ARG EXPOSE_PORT=8080

# Copy the statically-linked executable into the minimal scratch image
COPY --from=build [ \
    "${WORKDIR_ROOT}/${PROJECT_NAME}/target/${BUILD_TARGET}/${BUILD_MODE}/${BUILD_BIN}", \
    "./entrypoint" ]

EXPOSE ${EXPOSE_PORT}

VOLUME [ ${DATA_VOLUME} ]

# Bind the exposed port to Rocket that is used as the web framework
ENV ROCKET_PORT ${EXPOSE_PORT}

ENTRYPOINT [ "./entrypoint" ]
