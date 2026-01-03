# tectonic-api

[![API](https://img.shields.io/endpoint?url=https%3A%2F%2Fwinstxnhdw-tectonic-api.hf.space%2Fapi%2Fhealth&logo=huggingface&label=tectonic-api&labelColor=%230B0F19&color=brightgreen)](https://huggingface.co/spaces/winstxnhdw/tectonic-api)
[![build.yml](https://github.com/winstxnhdw/tectonic-api/actions/workflows/build.yml/badge.svg)](https://github.com/winstxnhdw/tectonic-api/actions/workflows/build.yml)
[![warmer.yml](https://github.com/winstxnhdw/tectonic-api/actions/workflows/warmer.yml/badge.svg)](https://github.com/winstxnhdw/tectonic-api/actions/workflows/warmer.yml)
[![dependabot.yml](https://github.com/winstxnhdw/tectonic-api/actions/workflows/dependabot.yml/badge.svg)](https://github.com/winstxnhdw/tectonic-api/actions/workflows/dependabot.yml)

[![Open in Spaces](https://huggingface.co/datasets/huggingface/badges/raw/main/open-in-hf-spaces-md-dark.svg)](https://huggingface.co/spaces/winstxnhdw/tectonic-api)
[![Open a Pull Request](https://huggingface.co/datasets/huggingface/badges/raw/main/open-a-pr-md-dark.svg)](https://github.com/winstxnhdw/tectonic-api/compare)

A simple [axum](https://github.com/tokio-rs/axum) API for compiling TeX/LaTeX with [Tectonic](https://github.com/tectonic-typesetting/tectonic), hosted on Hugging Face Spaces.

## Usage

Simply cURL the endpoint like in the following.

```bash
curl -O 'https://winstxnhdw-tectonic-api.hf.space/api/v1/compile' \
     -H 'Content-Type: application/json' \
     -d '{ "latex": "\\\documentclass{article}\\\begin{document}Hello, world!\\\end{document}" }'
```

## Development

You can spin the server up locally with the following.

```bash
docker build -f Dockerfile.build -t tectonic-api .
docker run --rm -e SERVER_PORT=7860 -p 7860:7860 tectonic-api
```
