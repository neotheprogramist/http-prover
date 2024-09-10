#!/usr/bin/env bash

set -eux
IMAGE_NAME="http-prover-test"
CONTAINER_ENGINE="${CONTAINER_ENGINE:-docker}"

# Check if the image already exists
if $CONTAINER_ENGINE images | grep -q "$IMAGE_NAME"; then
    echo "Image $IMAGE_NAME already exists. Skipping build step."
else
    echo "Image $IMAGE_NAME does not exist. Building the image..."

    if [ "${CI:-}" == "true" ]; then
        $CONTAINER_ENGINE buildx build -t $IMAGE_NAME . \
            --cache-from type=local,src=/tmp/.buildx-cache \
            --cache-to type=local,dest=/tmp/.buildx-cache-new,mode=max \
            --output=type=$CONTAINER_ENGINE,dest=image.tar
        $CONTAINER_ENGINE load -i image.tar
    else
        $CONTAINER_ENGINE build -t $IMAGE_NAME .
    fi

    if [ $? -ne 0 ]; then
        echo "Failed to build the image. Exiting."
        exit 1
    fi
fi

KEYGEN_OUTPUT=$(cargo run -p keygen)

PUBLIC_KEY=$(echo "$KEYGEN_OUTPUT" | grep "Public key" | awk '{print $3}' | tr -d ',' | tr -d '[:space:]')
PRIVATE_KEY=$(echo "$KEYGEN_OUTPUT" | grep "Private key" | awk '{print $3}' | tr -d ',' | tr -d '[:space:]')

KEYGEN_OUTPUT=$(cargo run -p keygen)

ADMIN_PUBLIC_KEY=$(echo "$KEYGEN_OUTPUT" | grep "Public key" | awk '{print $3}' | tr -d ',' | tr -d '[:space:]')
ADMIN_PRIVATE_KEY=$(echo "$KEYGEN_OUTPUT" | grep "Private key" | awk '{print $3}' | tr -d ',' | tr -d '[:space:]')

REPLACE_FLAG=""
if [ "$CONTAINER_ENGINE" == "podman" ]; then
    REPLACE_FLAG="--replace"
fi
$CONTAINER_ENGINE run -d --name http_prover_test $REPLACE_FLAG \
    -p 3040:3000 $IMAGE_NAME \
    --jwt-secret-key "secret" \
    --message-expiration-time 3600 \
    --session-expiration-time 3600 \
    --authorized-keys $PUBLIC_KEY,$ADMIN_PUBLIC_KEY \
    --admin-key $ADMIN_PUBLIC_KEY

PRIVATE_KEY=$PRIVATE_KEY PROVER_URL="http://localhost:3040" ADMIN_PRIVATE_KEY=$ADMIN_PRIVATE_KEY cargo test --no-fail-fast --workspace --verbose

$CONTAINER_ENGINE stop http_prover_test
$CONTAINER_ENGINE rm http_prover_test
