# Toadstool Evolution Handoff - February 9, 2026

**Team**: Toadstool (phase1/toadstool)
**Priority**: MEDIUM - Required for distributed GPU inference
**Codebase**: `ecoPrimals/phase1/toadstool/`

---

## Context

Toadstool is the compute primal. It manages GPU workloads, barraCUDA inference,
and compute delegation. Currently operational on both Tower (RTX 4070, 12GB VRAM)
and gate2 (RTX 3090, 24GB VRAM).

Full design: `docs/handoffs/TOADSTOOL_DISTRIBUTED_GPU_HANDOFF_FEB09_2026.md`

---

## HPC Resources

| Gate | GPU | VRAM | RAM | CPU | Status |
|------|-----|------|-----|-----|--------|
| Tower | RTX 4070 | 12 GB | 31 GB | 24 (i9-14900) | Toadstool running |
| gate2 | RTX 3090 | 24 GB | 251 GB | 128 (EPYC 9274F) | Toadstool running |
| **Total** | **2 GPUs** | **36 GB** | **282 GB** | **152** | |

---

## Evolution Items

### 1. GPU Job Queue (HIGH PRIORITY)

**What**: Accept compute jobs, queue them, execute on available GPU, return results.

**Methods**:
- `compute.submit(job)` - Submit a compute job (returns job_id)
- `compute.status(job_id)` - Check job status
- `compute.result(job_id)` - Retrieve completed result
- `compute.cancel(job_id)` - Cancel pending/running job
- `compute.list()` - List all jobs and their states

**Job types**:
- `inference`: Run model inference (Ollama, GGUF, safetensors)
- `transform`: Data transformation (embedding, tokenization)
- `custom`: Arbitrary compute via plugin

**Estimated**: 300 lines

### 2. Ollama Integration (MEDIUM PRIORITY)

**What**: Native Ollama management -- start/stop models, check availability, run inference.

**Methods**:
- `ollama.list_models()` - Models available on this gate
- `ollama.inference(model, prompt, params)` - Run inference
- `ollama.load(model)` - Preload model into VRAM
- `ollama.unload(model)` - Free VRAM

**Why**: Currently Squirrel talks to Ollama via Songbird HTTP proxy. Toadstool should
own model lifecycle management since it owns the GPU.

**Estimated**: 150 lines

### 3. Cross-Gate Compute Delegation (FUTURE)

**What**: Route compute jobs to the best available GPU across the mesh.

**How**:
- Plasmodium knows all gates and their GPU capabilities
- Job router: select gate by VRAM available, model already loaded, queue depth
- Forward job via Songbird mesh TCP relay
- Return result back through the mesh

**Example**: Gate2 (RTX 3090, 24GB) is better for large models. Tower (RTX 4070) is
better for quick inference. The router picks the right gate automatically.

**Estimated**: 500 lines (requires Songbird mesh relay to be active)

### 4. Multi-Family Socket Support (NEW)

**What**: Accept `--family-id` flag, create `toadstool-{family_id}.sock`.

**Estimated**: 10 lines

### 5. `discover_capabilities` JSON-RPC Method

**Response**:
```json
{
  "capabilities": [
    "compute.submit", "compute.status", "compute.result",
    "compute.cancel", "compute.list",
    "gpu.info", "gpu.memory",
    "ollama.list_models", "ollama.inference"
  ]
}
```

---

## Current Neural API Translations

Toadstool capabilities already registered in Neural API's translation table:
- `compute.submit` -> `toadstool.submit_job`
- `compute.status` -> `toadstool.job_status`
- `inference.run` -> `toadstool.run_inference`
- `gpu.info` -> `toadstool.gpu_info`
