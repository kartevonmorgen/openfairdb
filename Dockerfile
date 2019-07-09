# Dockerfile for creating a statically-linked Rust application using Docker's
# multi-stage build feature. This also leverages the docker build cache to
# avoid re-downloading dependencies if they have not changed between builds.
FROM clux/muslrust:nightly-2019-07-08 AS build

ARG WORKDIR_ROOT=/usr/src
ARG PROJECT_NAME=openfairdb

ARG BUILD_BIN=${PROJECT_NAME}
ARG BUILD_MODE=release
ARG BUILD_TARGET=x86_64-unknown-linux-musl

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
RUN cargo build --${BUILD_MODE} --target ${BUILD_TARGET} --bin ${BUILD_BIN}
# Delete all build artefacts that must(!) not be cached
RUN rm -f ./target/${BUILD_TARGET}/${BUILD_MODE}/${BUILD_BIN}*
RUN rm -f ./target/${BUILD_TARGET}/${BUILD_MODE}/deps/${BUILD_BIN}*
RUN rm -rf ./target/${BUILD_TARGET}/${BUILD_MODE}/.fingerprint/${BUILD_BIN}*

WORKDIR ${WORKDIR_ROOT}/${PROJECT_NAME}

# Copy all project (re-)sources the are required for building
COPY [ \
    "./migrations", \
    "./migrations/" ]
COPY [ \
    "./src", \
    "./src/" ]
COPY [ \
    "./openapi.yaml", \
    "./" ]

# Build the actual project
RUN cargo build --${BUILD_MODE} --target ${BUILD_TARGET} --bin ${BUILD_BIN}

# Strip debug symbols from the resulting executable
RUN strip ./target/${BUILD_TARGET}/${BUILD_MODE}/${BUILD_BIN}

###############################################################################

ARG DATA_VOLUME="/volume"

ARG EXPOSE_PORT=8080

###############################################################################

FROM scratch

# Copy the statically-linked executable into the minimal scratch image
COPY --from=build ${WORKDIR_ROOT}/${PROJECT_NAME}/target/${BUILD_TARGET}/${BUILD_MODE}/${BUILD_BIN} ./

EXPOSE ${EXPOSE_PORT}

VOLUME [ ${DATA_VOLUME} ]

# Bind the exposed port to Rocket that is used as the web framework
ENV ROCKET_PORT ${EXPOSE_PORT}

# Ensure that the name of the executable matches ${BUILD_BIN}!
CMD [ "./openfairdb" ]
