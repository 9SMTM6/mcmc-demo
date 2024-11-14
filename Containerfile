from docker.io/caddy:2-alpine

env CADDY_SERVE_ROOT="executable/dist/combined"

run mkdir -p ${CADDY_SERVE_ROOT}

arg PUBLIC_URL
env PUBLIC_URL=${PUBLIC_URL}

copy Caddyfile .
copy ${CADDY_SERVE_ROOT} ${CADDY_SERVE_ROOT}
# Ensure that the copied files were correctly prepared, by testing against filesize (and existence) of a sample file (currently ~1.2MB)
run SIZE_LIMIT=1200000 && FILE_LOC="${CADDY_SERVE_ROOT}/slim/mcmc-demo*_bg.wasm.br"\
    FILE_SIZE=$(stat -c %s ${FILE_LOC}) && \
    if [ "$FILE_SIZE" -gt "$SIZE_LIMIT" ]; then \
        echo "Error: ${FILE_LOC} exceeds the expected size. Did you run just caddy_prepare beforehand, or do you need to readjust the limits?" >&2; \
        exit 1; \
    fi

entrypoint [ "caddy", "run" ]
