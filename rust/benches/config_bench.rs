use criterion::{criterion_group, criterion_main, Criterion};
use github_user_fetcher::config::{Config, Mode};

fn bench_config_from_args(c: &mut Criterion) {
    c.bench_function("Config::from_args default CLI", |b| {
        b.iter(|| {
            let config = Config::from_args(std::env::args());
            assert!(matches!(config.mode, Mode::Cli));
        })
    });
}

criterion_group!(benches, bench_config_from_args);
criterion_main!(benches);
