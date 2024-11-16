FROM docker.io/caddy:2-alpine

ENV CADDY_SERVE_ROOT="executable/dist/combined"

RUN mkdir -p ${CADDY_SERVE_ROOT}

COPY Caddyfile .
COPY ${CADDY_SERVE_ROOT} ${CADDY_SERVE_ROOT}
# Ensure that the copied files were correctly prepared, by testing against filesize (and existence) of a sample file (currently ~1.2MB)
RUN SIZE_LIMIT=1200000 && FILE_LOC="${CADDY_SERVE_ROOT}/slim/mcmc-demo*_bg.wasm.br"\
    FILE_SIZE=$(stat -c %s ${FILE_LOC}) && \
    if [ "$FILE_SIZE" -gt "$SIZE_LIMIT" ]; then \
        echo "Error: ${FILE_LOC} exceeds the expected size. Did you run just caddy_prepare beforehand, or do you need to readjust the limits?" >&2; \
        exit 1; \
    fi

# ports are for: https/<3 https/3 (http-to-https-redirect or https/<3) http/3
EXPOSE 443/tcp 443/udp 80/tcp 80/udp

ARG PUBLIC_URL='http://localhost'
ENV PUBLIC_URL=${PUBLIC_URL}

CMD [ "caddy", "run" ]

# docker build . --build-arg PUBLIC_URL="http://localhost" -t mcmc-demo-webgpu
# sudo docker run --cap-add=NET_ADMIN -p 443:443 -p 443:443/udp -p 80:80 [-e PUBLIC_URL='http://mcmc-demo-webgpu.pages.dev'] mcmc-demo-webgpu
# https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-container-registry#labelling-container-images
# https://docs.github.com/en/packages/managing-github-packages-using-github-actions-workflows/publishing-and-installing-a-package-with-github-actions#publishing-a-package-using-an-action
# https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-container-registry#authenticating-in-a-github-actions-workflow
