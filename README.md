# faster-base64
Experimental Base64 implementation in Rust

## Features
* 1.8x faster encoding than `base64` package
* 2-4x faster decoding than `base64` package
* Auto-vectorization with `-C target-cpu=native`
* Zero-copy in-place decoding
