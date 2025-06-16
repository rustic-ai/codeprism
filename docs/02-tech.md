Modern large-context LLMs make code understanding mostly a *retrieval* problem.
The product below pairs an **incremental, polyglot call/data-flow graph** with an **MCP-compliant server** that streams only the files and ranges needed for each question.
It borrows Cline’s “structure-first, no index” philosophy — avoiding vector stores entirely — and relies on Tree-Sitter for ultra-fast re-parsing, Neo4j for graph queries, and Rust for safe, low-latency execution. The sections that follow spell out what the product does, how each component works, the data contracts that link them, and exactly how to implement and test a first version.

## Product specification

### Purpose

Give developers a chat interface that can **(a) explain, trace and visualise behaviour across any number of repositories and languages** and **(b) generate or patch code with the right surrounding context**, without ever bulk-indexing or leaking proprietary source.

### Key capabilities

| ID  | Capability        | Description                                                                                                                                       |
| --- | ----------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| F-1 | *Trace flow*      | From any symbol, file or log message, show the shortest or k-shortest path to another point in the system, crossing language and repo boundaries. |
| F-2 | *Explain node*    | Summarise what a function/route/query does using surrounding lines and inbound/outbound edges.                                                    |
| F-3 | *Patch safely*    | Accept model-generated edits, apply them via Git, and trigger an immediate graph refresh.                                                         |
| F-4 | *Live freshness*  | Graph updates < 250 ms after a developer saves a file.                                                                                            |
| F-5 | *Privacy control* | Only ranges the user explicitly approves are ever streamed to the LLM host.                                                                       |

### Non-functional requirements

* **Latency**: P99 end-to-end trace ≤ 1 s on a 5 MLOC mono-repo.
* **Language coverage**: JS/TS, Python, Java out-of-box; others via plug-in grammars.
* **Security**: MCP “consent-required” mode plus role-scoped tokens.
* **Extensibility**: new linkers (e.g. gRPC, Kafka topics) drop in without redeploying the core.

## Technical specification

### System overview

```
 Developer IDE / Claude ──MCP──▶ G-MCP Server ──Cypher──▶ Neo4j Graph
                                      ▲
                    file ranges ◀─────┘
                                      │
                         Kafka Δ bus ◀┐
                                      │ ast_patch
                         G-Core (Rust)│
          FS / Git hooks ──diff──────▶│ Tree-Sitter
```

### Components in detail

| Component              | Language            | Responsibilities                                                                                                                                     | Internals / key libraries                                                                                                                        | Notes                                                |
| ---------------------- | ------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------ | ---------------------------------------------------- |
| **G-Core**             | Rust                | Parse changed files, lift to *Universal-AST*, build/patch graph, emit `AstPatch` events.                                                             | `tree-sitter` & `tree-sitter-rust` bindings , snapshot diff engine, `rust-rdkafka` for Kafka                                                     | Runs as a single writer to avoid consistency issues. |
| **Graph Store**        | —                   | Persist nodes (`Function`, `Route`, `SQLQuery`, …) and edges (`CALLS`, `WRITES`, `EMITS`). Answer shortest-path, neighbourhood and subgraph queries. | Neo4j with Cypher  or TypeDB ; uses `SHORTEST` path queries                                                                                      | Sharded by repo when > 10 M nodes.                   |
| **G-MCP Server**       | Rust (Tokio + Axum) | Expose graph + code ranges via **Model Context Protocol** JSON-RPC over HTTP/SSE                                                                     | Implements `resources/list`, `resources/read`, `tools/trace_path`, `tools/explain_node`, `tools/neighbors`. Stores no state beyond an LRU cache. |                                                      |
| **Language Linkers**   | Rust plug-ins       | Add edges across languages: REST path ↔ controller, SQL string ↔ table, etc.                                                                         | Regex + small parsers (OpenAPI path precedence rules ).                                                                                          | Runs inside G-Core after AST build.                  |
| **IDE / Host plug-in** | TypeScript          | Calls MCP, builds prompts, applies Git patches.                                                                                                      | Official MCP client SDK                                                                                                                          | Cursor / Claude Desktop already speak MCP .          |

### Data contracts

*`AstPatch`* (Kafka topic)

```protobuf
message AstPatch {
  string repo;  string commit;
  repeated NodeAdd nodes_add;  repeated EdgeAdd edges_add;
  repeated string nodes_del;   repeated string edges_del;
}
```

*MCP tools*

| Tool           | Params                 | Returns                                      |
| -------------- | ---------------------- | -------------------------------------------- |
| `trace_path`   | `{from, to, maxHops?}` | `[{edge, node}]`, plus deduped snippet URIs. |
| `explain_node` | `{node}`               | Markdown summary + key in/out edges.         |
| `neighbors`    | `{node, direction}`    | List of adjacent node URIs.                  |

### Parsing pipeline (G-Core)

1. **Diff capture** – Git pre-commit & editor file-watcher supply changed ranges.
2. **Incremental parse** – Tree-Sitter updates the CST in ≤ 5 µs/LOC .
3. **U-AST projection** – Map CST symbols to language-neutral kinds.
4. **Edge extraction** – Walk tree; emit CALLS/WRITES/EMITS edges.
5. **Linker pass** – Extra scanners add cross-lang edges (REST, SQL, gRPC).
6. **Graph patch** – Merge via Neo4j Bolt driver; publish `AstPatch`.

### Security & privacy

* Server requires explicit `tools/call` consent per MCP spec .
* Snippet streaming enforces max-line and max-file policies; redaction hooks strip PII tokens.
* Role-scoped JWT: *graph-only*, *snippet-read*, *write-patch*.

### Observability

* Structured logs (OpenTelemetry JSON).
* Prometheus counters: parse time, patch size, MCP latency.
* Kafka dead-letter queue on failed `AstPatch`.

## Step-by-step implementation & testing (Rust version)

1. **Scaffold workspace**

   ```bash
   cargo new pags && cd pags
   cargo add tree-sitter tree-sitter-highlight neo4j bolt tokio axum
   cargo add rdkafka jsonrpc-core serde --features full
   ```
2. **Embed grammars** (`build.rs`) for `tree-sitter-{javascript,python,java}`.
3. **Write Universal-AST structs** (`ast/mod.rs`).
4. **Implement incremental parser**
   *Maintain a `HashMap<PathBuf, Tree>` and re-parse only changed bytes.*
5. **Edge extractor** (`analysis/extract.rs`)
   *Walk AST, collect calls/reads/writes; unit-test with golden fixtures.*
6. **Graph writer** (`storage/neo.rs`)
   *Batch `CREATE`/`MERGE` Cypher statements; verify idempotence with tests hitting a containerised Neo4j.*
7. **Kafka patch publisher** (`bus/mod.rs`)
   *Serialize `AstPatch` with `prost`; integration-test end-to-end via `docker-compose` bringing up Kafka.*
8. **MCP server** (`server/main.rs`)
   *Use Axum for HTTP + SSE.*
   *Implement JSON-RPC methods; back each tool call with Cypher queries (shortest path, etc.).*
9. **Snippet extractor**
   *Read file, clamp to `start-5…end+5` lines; highlight with `tree-sitter-highlight`.*
10. **Client smoke test**
    *Run `mcp-inspect` (CLI from SDK) and issue `trace_path` on a small sample repo.*
11. **Performance test**
    *Load Linux kernel (≈ 20 MLOC) into a Neo4j container; measure full parse time, then save one file and measure Δ parse + patch latency.*
12. **Security test**
    *Attempt unauthenticated `resources/read`; ensure 401.*
13. **CI pipeline**
    *GitHub Actions: build, run `cargo test`, spin up docker-compose for integration tests, archive coverage.*

---

With these specifications and steps you can stand up a fully functioning, MCP-enabled “chat with my multi-language codebase” assistant that remains accurate, fast and private as the repository evolves.
