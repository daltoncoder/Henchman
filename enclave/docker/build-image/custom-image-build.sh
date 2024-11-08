docker system prune -f
docker builder prune -f

read -p "Building which version [>= v0.5.0] : " codever

CODEVER=${codever:-v0.1.0}

docker build --rm --no-cache \
    -t tee_ai_agent:$CODEVER-$CHAIN \
    -t tee_ai_agent:latest \
    --build-arg UBUNTU_VERSION=22.04 \
    --build-arg CODE_VERSION=$CODEVER \
    .
