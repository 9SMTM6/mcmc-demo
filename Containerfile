FROM docker.io/joseluisq/static-web-server:2-alpine

RUN mkdir -p executable/dist/combined

RUN apk add --no-cache openssl

COPY container_entrypoint.sh /entrypoint.sh

RUN chmod +x /entrypoint.sh

VOLUME [ "/certs" ]

COPY sws.toml .
COPY executable/dist/combined executable/dist/combined
# Ensure that the copied files were correctly prepared, by testing against filesize (and existence) of a sample file (currently ~1.2MB)
RUN SIZE_LIMIT=1300000 && FILE_LOC="executable/dist/combined/slim/mcmc-demo*_bg.wasm.br"\
    FILE_SIZE=$(stat -c %s ${FILE_LOC}) && \
    if [ "$FILE_SIZE" -gt "$SIZE_LIMIT" ]; then \
        echo "Error: ${FILE_LOC} exceeds the expected size. Did you run just sws_prepare beforehand, or do you need to readjust the limits?" >&2; \
        exit 1; \
    fi

# ports are for: https/ http-to-https-redirect
EXPOSE 443/tcp 80/tcp

ENV TLS_HOST=localhost

CMD ["/entrypoint.sh"]

# docker build . -t mcmc-demo-webgpu
# sudo docker run --cap-add=NET_ADMIN -p 443:443 -p 80:80 mcmc-demo-webgpu
# https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-container-registry#labelling-container-images
# https://docs.github.com/en/packages/managing-github-packages-using-github-actions-workflows/publishing-and-installing-a-package-with-github-actions#publishing-a-package-using-an-action
# https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-container-registry#authenticating-in-a-github-actions-workflow
