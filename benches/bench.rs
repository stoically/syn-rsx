use criterion::{criterion_group, criterion_main, Criterion};
use quote::quote;

fn criterion_benchmark(c: &mut Criterion) {
    let tokens = quote! {
        <div>
            <hello world />
            <div>"String literal"</div>
            <tag-name attribute-key="value" />
            <tag:name attribute:key="value" />
            <tag::name attribute::key="value" />
            <input type="submit" />
            <div key=some::value() />
            <div>{ let block = "in node position"; }</div>
            <div { let block = "in attribute position"; } />
            <div key={ let block = "in attribute value position"; } />
        </div>
    };

    c.bench_function("syn_rsx::parse2", |b| {
        b.iter(|| syn_rsx::parse2(tokens.clone()))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
