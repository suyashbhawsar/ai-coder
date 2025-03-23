use ai_coder_interface_rs::utils::count_tokens;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

fn token_counting_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("token_counting");

    // Sample texts of various sizes
    let texts = [
        "Hello world",
        "This is a simple test for token counting benchmarks.",
        "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Suspendisse quis arcu et nisi tincidunt faucibus non eu nisi. Mauris lobortis tellus sit amet arcu tincidunt, in vulputate felis ullamcorper.",
        // Generate progressively larger texts
        std::iter::repeat("The quick brown fox jumps over the lazy dog. ")
            .take(10)
            .collect::<String>(),
        std::iter::repeat("The quick brown fox jumps over the lazy dog. ")
            .take(100)
            .collect::<String>(),
        std::iter::repeat("The quick brown fox jumps over the lazy dog. ")
            .take(1000)
            .collect::<String>(),
    ];

    for (i, text) in texts.iter().enumerate() {
        group.bench_with_input(BenchmarkId::new("text", i), text, |b, text| {
            b.iter(|| count_tokens(text));
        });
    }

    group.finish();
}

criterion_group!(benches, token_counting_benchmark);
criterion_main!(benches);
