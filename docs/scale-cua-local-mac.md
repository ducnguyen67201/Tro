# ScaleCUA local Mac development spike

This is an unpromoted Apple Silicon experiment, not the production serving path. Tro's initial quality recommendation remains direct OpenAI Responses with `gpt-5.6`. The supported ScaleCUA candidate is the research checkpoint `extreme1228/ScaleCUA-qwen3.5-osworld`; vLLM and MLX-VLM are servers, not model names and are not available through OpenRouter.

## Frozen tuple

| Component              | Pin                                                                        | Status                            |
| ---------------------- | -------------------------------------------------------------------------- | --------------------------------- |
| ScaleCUA weights       | revision `2ad1314b6076591e35f299b2efed214e2454deab`                        | Research-use/license review open  |
| ScaleCUA tool behavior | upstream commit `3929e2fe364623153f2caa94ead71dc1aea50fb0`                 | Adapter protocol pinned           |
| MLX-VLM                | tag/package `0.6.8`, tag commit `61990c9054f2bc7bb8f32541e3238b4a58fe64e5` | Conversion compatibility unproven |
| Target machine         | M4 Max, 48 GB unified memory                                               | Supervised spike pending          |

## Isolated setup

Use a fresh virtual environment and a dedicated cache. Review the pinned checkpoint's remote code before enabling it.

```bash
python3 -m venv .venv-mlx-scalecua
source .venv-mlx-scalecua/bin/activate
python -m pip install --upgrade pip
python -m pip install 'mlx-vlm==0.6.8' 'huggingface-hub>=0.30,<1'
export SCALE_CUA_API_KEY='replace-with-a-long-random-local-secret'
export HF_HOME="$PWD/.cache/scale-cua-mlx"
mlx_vlm.server \
  --host 127.0.0.1 \
  --port 8000 \
  --api-key "$SCALE_CUA_API_KEY" \
  --model extreme1228/ScaleCUA-qwen3.5-osworld \
  --trust-remote-code
```

Configure only a development API process:

```bash
COMPUTER_PROVIDER=scale_cua
SCALE_CUA_BASE_URL=http://127.0.0.1:8000/v1
SCALE_CUA_MODEL=scalecua
SCALE_CUA_API_KEY="$SCALE_CUA_API_KEY"
SCALE_CUA_ALLOWED_HOST=
```

Run `cargo test -p api --test scale_cua --locked` before sending a fixture. The spike must then prove all of the following without personal screens:

- the pinned checkpoint converts/loads and the served model is explicitly named `scalecua`;
- one image plus the pinned native `computer` tool yields exactly one parseable call;
- Vietnamese input survives without mojibake;
- a 20-turn fixture run stays within the recorded memory budget;
- latency, peak memory, quantization, semantic/visual action rate, and fixture score are recorded as a distinct tuple;
- the messaging fixture meets the promotion thresholds in `docs/release.md`.

Stop the server, remove the virtual environment/cache, and delete any local screen-test artifacts after the spike. If conversion, tool behavior, Vietnamese text, memory, or quality fails, do not ship the MLX route; keep the vLLM canary as the only ScaleCUA path. A lower-bit model that fits is not considered equivalent and requires its own evaluation tuple.
