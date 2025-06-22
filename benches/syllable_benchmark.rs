use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use vi::Syllable;

fn benchmark_syllable_push(c: &mut Criterion) {
    c.bench_function("syllable_push_single_char", |b| {
        b.iter(|| {
            let mut syllable = Syllable::default();
            syllable.push(black_box('v'));
            black_box(syllable);
        })
    });

    c.bench_function("syllable_push_word_building", |b| {
        b.iter(|| {
            let mut syllable = Syllable::default();
            for ch in black_box("viet".chars()) {
                syllable.push(ch);
            }
            black_box(syllable.to_string());
        })
    });
}

criterion_group!(benches, benchmark_syllable_push);
criterion_main!(benches);
