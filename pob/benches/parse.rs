use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pob::{RoXmlPathOfBuilding, SerdePathOfBuilding};

static V316_EMPTY: &str = include_str!("../test/316_empty.xml");
static V316_POISON_OCC: &str = include_str!("../test/316_poison_occ.xml");

fn parse_roxmltree(xml: &str) -> RoXmlPathOfBuilding<'_> {
    RoXmlPathOfBuilding::from_xml(xml).unwrap()
}

fn parse_serde(xml: &str) -> SerdePathOfBuilding {
    SerdePathOfBuilding::from_xml(xml).unwrap()
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("parse_roxmltree", |b| b.iter(|| parse_roxmltree(black_box(V316_POISON_OCC))));
    c.bench_function("parse_serde", |b| b.iter(|| parse_serde(black_box(V316_POISON_OCC))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

