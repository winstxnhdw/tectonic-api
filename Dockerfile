FROM ghcr.io/winstxnhdw/tectonic-api:main

ENV SERVER_PORT 7860
ENV XDG_CACHE_HOME /cache

RUN mkdir -p /target /cache/Tectonic
RUN chmod 777 /target
RUN chmod -R 777 /cache
