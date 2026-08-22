FROM ghcr.io/winstxnhdw/tectonic-api:main

ENV SERVER_PORT=7860
ENV CONSUL_SERVICE_ADDRESS=winstxnhdw-tectonic-api.hf.space

EXPOSE $SERVER_PORT
