//! Benchmarks for renderer crate

use archflow_renderer::{FontManager, TextRenderer, TextStyle};
use criterion::{Criterion, black_box, criterion_group, criterion_main};

fn criterion_config() -> Criterion {
    Criterion::default()
        .sample_size(50)
        .measurement_time(std::time::Duration::from_secs(2))
}

fn bench_font_manager(c: &mut Criterion) {
    c.bench_function("font_manager_new", |b| b.iter(|| FontManager::new()));
}

fn bench_text_buffer_creation(c: &mut Criterion) {
    let mut renderer = TextRenderer::new();

    c.bench_function("text_buffer_short", |b| {
        b.iter(|| renderer.create_text_buffer("Hello"))
    });

    c.bench_function("text_buffer_medium", |b| {
        b.iter(|| renderer.create_text_buffer("This is a medium length text"))
    });

    c.bench_function("text_buffer_long", |b| {
        b.iter(|| {
            renderer.create_text_buffer("This is a much longer text that contains more words and should take longer to process")
        })
    });

    c.bench_function("text_buffer_multiline", |b| {
        b.iter(|| renderer.create_text_buffer("Line 1\nLine 2\nLine 3\nLine 4\nLine 5"))
    });
}

fn bench_text_style(c: &mut Criterion) {
    let mut renderer = TextRenderer::new();

    c.bench_function("text_style_default", |b| {
        b.iter(|| renderer.create_text_buffer_with_style("Test", TextStyle::default()))
    });

    c.bench_function("text_style_custom", |b| {
        b.iter(|| {
            let style = TextStyle {
                font_size: 24.0,
                font_family: "serif".to_string(),
                ..TextStyle::default()
            };
            renderer.create_text_buffer_with_style("Styled text", style)
        })
    });
}

fn bench_buffer_updates(c: &mut Criterion) {
    let mut renderer = TextRenderer::new();
    let mut buffer = renderer.create_text_buffer("Original text");

    c.bench_function("text_buffer_update", |b| {
        b.iter(|| {
            renderer.update_text_buffer(&mut buffer, "Updated text");
        })
    });

    c.bench_function("text_style_update", |b| {
        b.iter(|| {
            let new_style = TextStyle {
                font_size: 32.0,
                ..TextStyle::default()
            };
            renderer.update_text_style(&mut buffer, new_style);
        })
    });
}

fn bench_buffer_dimensions(c: &mut Criterion) {
    let mut renderer = TextRenderer::new();
    let buffer = renderer.create_text_buffer("Measure this text");

    c.bench_function("buffer_dimensions", |b| {
        b.iter(|| renderer.buffer_dimensions(black_box(&buffer)))
    });
}

criterion_group!(
    name = renderer_benches;
    config = criterion_config();
    targets = bench_font_manager, bench_text_buffer_creation, bench_text_style, bench_buffer_updates, bench_buffer_dimensions
);

criterion_main!(renderer_benches);
