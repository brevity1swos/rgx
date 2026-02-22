use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rgx::engine::{create_engine, EngineFlags, EngineKind};

fn bench_compile_and_match(c: &mut Criterion) {
    let pattern = r"(\w+)@(\w+)\.(\w+)";
    let text = "user@example.com admin@test.org hello@world.net";
    let flags = EngineFlags::default();

    let mut group = c.benchmark_group("engine_compile_match");

    group.bench_function("rust_regex", |b| {
        let engine = create_engine(EngineKind::RustRegex);
        b.iter(|| {
            let compiled = engine.compile(black_box(pattern), &flags).unwrap();
            compiled.find_matches(black_box(text)).unwrap()
        });
    });

    group.bench_function("fancy_regex", |b| {
        let engine = create_engine(EngineKind::FancyRegex);
        b.iter(|| {
            let compiled = engine.compile(black_box(pattern), &flags).unwrap();
            compiled.find_matches(black_box(text)).unwrap()
        });
    });

    group.finish();
}

criterion_group!(benches, bench_compile_and_match);
criterion_main!(benches);
