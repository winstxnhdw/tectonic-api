FROM ghcr.io/winstxnhdw/tectonic-api:main

ENV SERVER_PORT 7860

RUN mkdir /target
RUN chmod 777 /target
