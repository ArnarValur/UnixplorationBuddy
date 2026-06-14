# Elite Dangerous Companion Brain — Feasibility Report

> Researched: 2026-06-11
> Hardware: Saturn (ASUS Ascent GX10, GB10, 128GB unified, ARM64)

---

## 1. Saturn Hardware: What Can GB10 Actually Run?

| Feature | Value |
|---|---|
| **Architecture** | Grace Blackwell (ARM64: 10× Cortex-X925 + 10× Cortex-A725) |
| **GPU** | Blackwell-gen, 6,144 CUDA cores, 5th-gen Tensor Cores (FP4) |
| **Memory** | 128 GB unified LPDDR5X (~273 GB/s bandwidth) |
| **AI Perf** | ~1 PFLOP FP4 (sparse) |
| **Key Feature** | CPU and GPU share the same 128GB pool — no VRAM/RAM distinction |

### Training Feasibility
- **3B model full fine-tune**: ✅ Easily viable — fits comfortably in 128GB unified memory
- **7B model LoRA/QLoRA**: ✅ Viable — QLoRA (4-bit) keeps the 7B model ~4GB + gradients/optimizer states well within budget
- **7B model full fine-tune**: ⚠️ Tight but possible — needs ~56GB for model + optimizer states in bf16
- **13B+ model**: ⚠️ QLoRA only, full fine-tune won't fit

### Inference Performance (estimated for Saturn)
| Model Size | Quantization | Expected tok/s (generation) |
|---|---|---|
| 3B | Q4_K_M | ~150-200 tok/s |
| 7B | Q4_K_M | ~80-120 tok/s |
| 7B | FP16 | ~40-60 tok/s |
| 26B (Gemma4) | Q4_K_M | ~58 tok/s (community-reported) |

### ARM64 Ecosystem
- PyTorch: ✅ Fully supported via NVIDIA Docker containers
- llama.cpp: ✅ Compiles from source with `-DGGML_CUDA=ON`, needs `-DCMAKE_CUDA_ARCHITECTURES=121`
- Ollama: ✅ Works on DGX Spark
- Unsloth: ✅ Has explicit DGX Spark/Blackwell support with optimized Triton kernels
- **Recommendation**: Use Docker containers for training, native llama.cpp for inference

---

## 2. Model Selection

### Top Recommendations (Ranked)

| Model | Params | Why for This Project |
|---|---|---|
| **Gemma 3 4B** ⭐ | 4B | Best JSON/structured output (100% parse rates), 131K context, great for tool-use patterns |
| **Llama 3.2 3B** | 3B | Best ecosystem/tooling, Unsloth-optimized, fastest to get started |
| **Phi-4 mini** | ~3.8B | Best reasoning per parameter — ideal for analytical tasks |
| **Gemma 4 E4B** | 4B | Newest (April 2026), improved reasoning over Gemma 3, worth evaluating |
| **Mistral 7B** | 7B | Strong general-purpose, but larger = slower for TUI use |

### Non-LLM Approaches (Per Use Case)
| Approach | Use Case | Size | Notes |
|---|---|---|---|
| **XGBoost/LightGBM classifier** | Bio prediction | <1MB | Learn boundary refinements beyond static rules |
| **Sentence-transformer embedding** | System name similarity, lore search | ~80MB (MiniLM-L6) | ONNX inference in Rust via `ort` crate |
| **Regression model** | Value estimation refinement | <1MB | Deterministic formulas already sufficient — ML adds nothing |

---

## 3. Elite Dangerous Data Sources

### Structured Data
| Source | Format | Size | Content |
|---|---|---|---|
| **EDSM Nightly Dumps** | JSON.gz | ~2 GB compressed | System coordinates, body data, full galaxy |
| **Spansh Dumps** | JSON.gz | Multi-GB | Systems, bodies, factions, stations — nightly |
| **Canonn Bioforge/codex.json.gz** | JSON | Variable | Bio/geo/xeno spawn conditions, species boundaries |
| **Player Journals** | JSON Lines | ~5-50 MB per session | Real-time gameplay events |
| **EDDN** | ZeroMQ stream (JSON) | Real-time | Live event relay from all connected players |

### Lore/Text Data
| Source | Format | Access | Est. Size |
|---|---|---|---|
| **Galnet Articles** | JSON via API | `https://cms.zaonce.net/en-GB/jsonapi/node/galnet_article` | ~3,000+ articles, ~2-5 MB text |
| **Elite Dangerous Wiki** | HTML | Scrape or API | ~10,000+ pages, ~50-100 MB text |
| **Codex entries (in-game)** | JSON (journals) | Extract from player journals | ~500 entries |

### Training Corpus Estimates
- **Lore/commentary fine-tune**: ~100-200 MB clean text
- **Structured data fine-tuning**: 2-10 GB structured JSON

---

## 4. Use Cases & Architecture

| Use Case | Best ML Approach | Why |
|---|---|---|
| **Bio prediction refinement** | Gradient-boosted classifier (XGBoost) | Learn subtle multi-variable correlations rules miss. Lightweight, runs natively in Rust. |
| **System naming patterns** | Embedding model (MiniLM via ONNX) | "Find systems similar to X" — vector nearest-neighbor. |
| **Route intelligence** | RAG over system/body database | "What interesting systems near my route?" — strongest LLM use case. |
| **Lore/commentary** | Fine-tuned small LLM (Gemma 3 4B / Llama 3.2 3B) | "Tell me about this system's history" — fine-tune on Galnet + Wiki. |
| **Value estimation** | None needed ❌ | Deterministic formulas already exact. |
| **Real-time companion** | Fine-tuned LLM + RAG hybrid | The flagship feature. |

### Recommended Hybrid Architecture
```
┌─────────────────────────────────────┐
│ UnixplorationBuddy (Rust TUI)       │
├─────────────┬───────────────────────┤
│ Fast Path   │ Slow Path             │
│ (embedded)  │ (async, background)   │
├─────────────┼───────────────────────┤
│ Bio predict │ LLM companion queries │
│ (XGBoost)   │ (Ollama HTTP API)     │
│ Value calc  │ Lore RAG search       │
│ (formulas)  │ (Embedding + vector)  │
└─────────────┴───────────────────────┘
```

---

## 5. Training Pipeline

### QLoRA Fine-Tuning Config
```
Hardware: Saturn (GB10)
Framework: Unsloth (recommended for Blackwell) or HuggingFace transformers
Method: QLoRA (4-bit quantization + LoRA adapters)
  - Rank: 32-64
  - Alpha: 64-128
  - Target modules: q_proj, v_proj, k_proj, o_proj
Base model: Gemma 3 4B (or Llama 3.2 3B)
Dataset: ~10-50K instruction pairs
Training time estimate: 2-8 hours on Saturn
```

### Export
1. Export fine-tuned model to GGUF format
2. Quantize to Q4_K_M (best quality/speed tradeoff)
3. Serve with Ollama on Saturn

---

## 6. Integration with UnixplorationBuddy

### Recommended: Ollama HTTP API
```
UnixplorationBuddy (Rust TUI) ←→ HTTP/REST ←→ Ollama (localhost:11434)
```
- Ollama as systemd service on Saturn
- TUI calls via `ureq` (already a dependency)
- Background thread, stream tokens to companion panel
- Never blocks TUI main loop

### Latency Expectations
| Operation | Expected Latency | TUI Impact |
|---|---|---|
| Bio prediction (existing rules) | <1ms | ✅ Instant |
| Bio prediction (XGBoost) | <5ms | ✅ Instant |
| Embedding lookup (ONNX) | 10-50ms | ✅ Imperceptible |
| LLM short response (3B Q4) | 200-500ms first token | ⚠️ Async panel |
| LLM long response (3B Q4) | 1-5 seconds total | ⚠️ Stream tokens |

### Rust Dependencies
```toml
[dependencies]
ort = "2.0"        # ONNX Runtime (embedding model)
tokenizers = "0.21" # HuggingFace tokenizer
ndarray = "0.16"   # Tensor math
# ureq already present for HTTP calls to Ollama
```

---

## Phased Plan

### Phase 0: Foundation (2 weeks)
- [ ] Set up Python/Docker ML environment on Saturn
- [ ] Scrape Galnet corpus, download Spansh dumps
- [ ] Create instruction-tuning dataset (1K-5K examples)
- [ ] No changes to UnixplorationBuddy yet

### Phase 1: Proof of Concept (Weeks 3-4)
- [ ] Fine-tune Llama 3.2 3B on ED lore using Unsloth/QLoRA on Saturn
- [ ] Export to GGUF Q4_K_M, test with Ollama
- [ ] Evaluate: fine-tuned vs base + system prompt

### Phase 2: Integration MVP (Weeks 5-6)
- [ ] Add Ollama HTTP client to UnixplorationBuddy
- [ ] Create companion panel in TUI for LLM commentary
- [ ] Basic queries: "Describe this system" on system entry
- [ ] Background thread, streamed tokens

### Phase 3: RAG & Enrichment (Weeks 7-8)
- [ ] Add MiniLM embedding model via `ort` crate
- [ ] Build vector index of Galnet + Wiki excerpts
- [ ] RAG pipeline: system context → retrieve lore → LLM
- [ ] Evaluate XGBoost bio classifier vs current rules

### Phase 4: Polish (Weeks 9+)
- [ ] Compare Gemma 3 4B vs Llama 3.2 3B fine-tuned
- [ ] Optimize quantization levels
- [ ] Consider embedded llama.cpp if Ollama overhead noticeable
- [ ] Scale training data (EDDN historical, larger journals)

---

## Key Risks

| Risk | Mitigation |
|---|---|
| Model hallucinates ED facts | RAG grounding + "I don't know" training examples |
| ARM64 compilation issues | Docker containers; NVIDIA pre-built images |
| Too slow for TUI | 3B Q4 model; async panel; never block main loop |
| Training data quality | Start small with Galnet; iterate |
| Scope creep | Phase 1 = fine-tuned model answering questions, nothing more |

---

## Bottom Line

**Saturn can absolutely do this.** The GB10 with 128GB unified memory can fine-tune a 3B-4B model in hours and run inference at 100+ tok/s. The biggest effort is data curation, not compute.

**Start with**: Llama 3.2 3B + QLoRA + Galnet corpus → Ollama on Saturn → `ureq` HTTP from UnixplorationBuddy.
