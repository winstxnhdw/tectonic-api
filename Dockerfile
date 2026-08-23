FROM ghcr.io/winstxnhdw/tectonic-api:main

ENV SERVER_PORT=7860
ENV OTEL_SEMCONV_STABILITY_OPT_IN=http
ENV CONSUL_SERVICE_ADDRESS=winstxnhdw-tectonic-api.hf.space

EXPOSE $SERVER_PORT
