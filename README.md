# tectonic-api

[![main.yml](https://github.com/winstxnhdw/tectonic-api/actions/workflows/main.yml/badge.svg)](https://github.com/winstxnhdw/tectonic-api/actions/workflows/main.yml)
[![build.yml](https://github.com/winstxnhdw/tectonic-api/actions/workflows/build.yml/badge.svg)](https://github.com/winstxnhdw/tectonic-api/actions/workflows/build.yml)
[![warmer.yml](https://github.com/winstxnhdw/tectonic-api/actions/workflows/warmer.yml/badge.svg)](https://github.com/winstxnhdw/tectonic-api/actions/workflows/warmer.yml)
[![dependabot.yml](https://github.com/winstxnhdw/tectonic-api/actions/workflows/dependabot.yml/badge.svg)](https://github.com/winstxnhdw/tectonic-api/actions/workflows/dependabot.yml)

A simple [axum](https://github.com/tokio-rs/axum) API for compiling TeX/LaTeX with [Tectonic](https://github.com/tectonic-typesetting/tectonic), hosted on Hugging Face Spaces.

## Usage

Simply cURL the endpoint like in the following.

```bash
curl -O 'https://winstxnhdw-tectonic-api.hf.space/v1/compile' \
  -H 'Content-Type: application/json' \
  -d '{
  "latex": "\\\documentclass{article}\\\begin{document}Hello, world!\\\end{document}"
}'
```

## Development

You can spin the server up locally with the following.

```bash
docker build -f Dockerfile.build -t tectonic-api .
docker run --rm -e SERVER_PORT=5000 -p 5000:5000 tectonic-api
```
