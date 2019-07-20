# Dockerfile for creating a statically-linked Rust application using Docker's
# multi-stage build feature. This also leverages the docker build cache to
# avoid re-downloading dependencies if they have not changed between builds.


###############################################################################
# Define global ARGs for all stages

ARG WORKDIR_ROOT=/usr/src

ARG PROJECT_NAME=openfairdb

ARG BUILD_TARGET=x86_64-unknown-linux-musl

ARG BUILD_MODE=release

ARG BUILD_BIN=${PROJECT_NAME}


###############################################################################
# 1st Build Stage
FROM clux/muslrust:nightly-2019-07-08 AS build

# Import global ARGs
ARG WORKDIR_ROOT
ARG PROJECT_NAME
ARG BUILD_TARGET
ARG BUILD_MODE
ARG BUILD_BIN

WORKDIR ${WORKDIR_ROOT}

# Docker build cache: Create and build an empty dummy project with all
# external dependencies to avoid redownloading them on subsequent builds
# if unchanged.
RUN USER=root cargo new --bin ${PROJECT_NAME}
WORKDIR ${WORKDIR_ROOT}/${PROJECT_NAME}
COPY [ \
    "Cargo.toml", \
    "Cargo.lock", \
    "./" ]
# Build the dummy project(s), then delete all build artefacts that must(!) not be cached
RUN cargo build --${BUILD_MODE} --target ${BUILD_TARGET} --all \
    && \
    rm -f ./target/${BUILD_TARGET}/${BUILD_MODE}/${PROJECT_NAME}* \
    && \
    rm -f ./target/${BUILD_TARGET}/${BUILD_MODE}/deps/${PROJECT_NAME}* \
    && \
    rm -rf ./target/${BUILD_TARGET}/${BUILD_MODE}/.fingerprint/${PROJECT_NAME}*

# Copy all project (re-)sources that are required for building
COPY [ \
    "./migrations", \
    "./migrations/" ]
COPY [ \
    "./src", \
    "./src/" ]
COPY [ \
    "./openapi.yaml", \
    "./" ]

# Test and build the actual project
RUN cargo test --${BUILD_MODE} --target ${BUILD_TARGET} --all \
    && \
    cargo build --${BUILD_MODE} --target ${BUILD_TARGET} --bin ${BUILD_BIN} \
    && \
    strip ./target/${BUILD_TARGET}/${BUILD_MODE}/${BUILD_BIN}


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
