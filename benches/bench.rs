use criterion::{criterion_group, criterion_main, Criterion};
use quote::quote;

fn criterion_benchmark(c: &mut Criterion) {
    let tokens = quote! {
        <div>
            <button onclick=do_something()>"hi"</button>
            <p>{ value }</p>
        </div>
    };

    c.bench_function("syn_rsx::parse2", |b| {
        b.iter(|| syn_rsx::parse2(tokens.clone()))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
