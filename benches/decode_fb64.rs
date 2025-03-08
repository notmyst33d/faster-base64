use std::fs::File;
use std::io::Read;
use base64::prelude::*;

fn main() {
    divan::main();
}

#[divan::bench(sample_count = 16, sample_size = 1)]
fn benchmark(b: divan::Bencher) {
    const MB_SIZE: usize = 1024;
    let buf_size = MB_SIZE * 1024 * 1024;

    let mut buf = vec![0u8; buf_size];

    let mut f = File::open("/dev/urandom").unwrap();
    f.read_exact(&mut buf).unwrap();

    let mut bytes = BASE64_STANDARD.encode(&buf).into_bytes();
    b.bench_local(move || {
        faster_base64::decode(&mut bytes);
    });
}
