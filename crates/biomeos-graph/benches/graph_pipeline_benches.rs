// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, reason = "benchmarks use unwrap for setup")]

//! Graph ingestion benchmarks
//!
//! Measures performance of TOML graph parsing — the critical ingestion
//! path that runs on every graph load and deployment execution.

use criterion::{Criterion, black_box, criterion_group, criterion_main};

const SMALL_GRAPH: &str = r#"
[graph]
name = "bench-small"
version = "1.0.0"
description = "Small benchmark graph"
coordination = "Sequential"

[[nodes]]
id = "start"
operation = "primal.health"
capability = "health"

[[nodes]]
id = "deploy"
operation = "primal.start"
capability = "lifecycle"
depends_on = ["start"]
"#;

const MEDIUM_GRAPH: &str = r#"
[graph]
name = "bench-medium"
version = "1.0.0"
description = "Medium benchmark graph with parallel phases"
coordination = "Parallel"

[[nodes]]
id = "health-check"
operation = "primal.health"
capability = "health"

[[nodes]]
id = "discover-crypto"
operation = "primal.discover"
capability = "crypto"
depends_on = ["health-check"]

[[nodes]]
id = "discover-mesh"
operation = "primal.discover"
capability = "discovery"
depends_on = ["health-check"]

[[nodes]]
id = "discover-compute"
operation = "primal.discover"
capability = "compute"
depends_on = ["health-check"]

[[nodes]]
id = "start-crypto"
operation = "primal.start"
capability = "crypto"
depends_on = ["discover-crypto"]

[[nodes]]
id = "start-mesh"
operation = "primal.start"
capability = "discovery"
depends_on = ["discover-mesh"]

[[nodes]]
id = "start-compute"
operation = "primal.start"
capability = "compute"
depends_on = ["discover-compute"]

[[nodes]]
id = "verify"
operation = "primal.verify"
capability = "health"
depends_on = ["start-crypto", "start-mesh", "start-compute"]

[[edges]]
from = "health-check"
to = "discover-crypto"
edge_type = "DependsOn"

[[edges]]
from = "health-check"
to = "discover-mesh"
edge_type = "DependsOn"

[[edges]]
from = "health-check"
to = "discover-compute"
edge_type = "DependsOn"
"#;

fn bench_parse_small_graph(c: &mut Criterion) {
    c.bench_function("graph_parse_small_2node", |b| {
        b.iter(|| {
            biomeos_graph::parser::GraphParser::parse_toml(black_box(SMALL_GRAPH)).unwrap();
        });
    });
}

fn bench_parse_medium_graph(c: &mut Criterion) {
    c.bench_function("graph_parse_medium_8node", |b| {
        b.iter(|| {
            biomeos_graph::parser::GraphParser::parse_toml(black_box(MEDIUM_GRAPH)).unwrap();
        });
    });
}

fn bench_parse_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("graph_parse_throughput");
    group.throughput(criterion::Throughput::Bytes(MEDIUM_GRAPH.len() as u64));
    group.bench_function("medium_bytes_per_sec", |b| {
        b.iter(|| {
            biomeos_graph::parser::GraphParser::parse_toml(black_box(MEDIUM_GRAPH)).unwrap();
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_parse_small_graph,
    bench_parse_medium_graph,
    bench_parse_throughput,
);
criterion_main!(benches);
