use criterion::{criterion_group, criterion_main, Criterion};
use github_user_fetcher::user::{fetch_github_user, set_base_url_override, User};

mod bench_utils;
use bench_utils::silence_stdout;

#[path = "../tests/utils/mock_server.rs"] 
mod mock_server;

fn bench_fetch_github_user(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mock_server = rt.block_on(mock_server::setup());
    let client = reqwest::Client::builder()
        .user_agent("rust-poem-github-client")
        .build()
        .unwrap();

    set_base_url_override(Some(mock_server.uri()));

    c.bench_function("fetch_github_user async (mocked)", |b| {
        b.to_async(&rt).iter(|| async {
            fetch_github_user(&client, "octocat").await.unwrap();
        });
    });

    set_base_url_override(None);
}

fn bench_user_print(c: &mut Criterion) {
    let user = User {
        login: "octocat".into(),
        name: Some("The Octocat".into()),
        company: Some("GitHub".into()),
        location: Some("San Francisco".into()),
    };

    c.bench_function("user print", |b| {
        b.iter(|| silence_stdout(|| user.print()));
    });
}

criterion_group!(benches, bench_fetch_github_user, bench_user_print);
criterion_main!(benches);
