# ScaleCUA vLLM canary

This optional profile is isolated from Tro's Postgres/default stack. It is a research/canary serving candidate for Linux with NVIDIA GPUs, not an automatic fallback and not a production approval.

## Frozen inputs

- vLLM `0.26.0`, multi-architecture image digest `sha256:ffb2d59b1c059a5bd8d781320c9f5189de8293693b7d95da54befddaa54abf52` (amd64 child digest `sha256:770fe65b2c73ee74a5c42165cf3433de4048cc2cd9c57a937ca4e35aba5aa87b`)
- checkpoint `extreme1228/ScaleCUA-qwen3.5-osworld`, revision `2ad1314b6076591e35f299b2efed214e2454deab`
- ScaleCUA runtime/schema reference `3929e2fe364623153f2caa94ead71dc1aea50fb0`
- served name `scalecua`, BF16, max model length 16,384, GPU memory utilization 0.90

The registry digest and model revision were resolved during implementation. They have not passed the GPU/30-run promotion gates yet; do not relabel this tuple as production evidence.

## Start and verify

```bash
export SCALE_CUA_API_KEY='replace-with-a-long-random-secret'
docker compose -f deploy/scale-cua/docker-compose.yml --profile scale-cua config
docker compose -f deploy/scale-cua/docker-compose.yml --profile scale-cua up -d
curl --fail --silent \
  -H "Authorization: Bearer $SCALE_CUA_API_KEY" \
  http://127.0.0.1:8000/v1/models
cargo test -p api --test scale_cua --locked
```

The port is bound to numeric loopback. Keep public ingress/firewall rules closed. Do not add `--allowed-local-media-path`, redirects, unpinned images, or an OpenRouter route. `--trust-remote-code` remains enabled only because the checkpoint requires the reviewed pinned revision; re-review it when the revision changes.

If vLLM 0.26.0 proves incompatible, test only 0.25.1, 0.25.0, then 0.24.0, newest first, and record the failure. A new image, revision, dtype, quantization, context length, or GPU is a new evaluation tuple.

## License decision checklist

Production promotion remains closed until an owner records and signs all items:

- checkpoint research-use terms and commercial-use permission;
- inherited Qwen3.5-9B base-model license and acceptable-use terms;
- ScaleCUA repository/evaluator code licenses;
- vLLM image/dependencies and any redistribution obligations;
- whether weights or remote code will be redistributed or only privately hosted;
- customer disclosure for the selected data recipient and retention boundary;
- legal approval for commercial use, pilot scope, and model-output limitations.
