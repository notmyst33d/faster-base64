mod tables;

use crate::tables::*;

pub fn encode(data: &[u8]) -> String {
    let len = data.len();
    let data_ptr = data.as_ptr();
    let mut sb = String::with_capacity(len);
    let mut i = 0;
    loop {
        if i > len - 1 {
            break;
        }
        let available_bytes = len - i;
        let mut v = (unsafe { (data_ptr.add(i) as *const u32).read() }).swap_bytes() as usize;
        if available_bytes == 2 {
            v &= 0xffff0000;
            sb.push_str(TABLE[v >> 26 & 0x3f]);
            sb.push_str(TABLE[v >> 20 & 0x3f]);
            sb.push_str(TABLE[v >> 14 & 0x3f]);
            sb.push_str("=");
            break;
        } else if available_bytes == 1 {
            v &= 0xff000000;
            sb.push_str(TABLE[v >> 26 & 0x3f]);
            sb.push_str(TABLE[v >> 20 & 0x3f]);
            sb.push_str("=");
            sb.push_str("=");
            break;
        } else {
            sb.push_str(TABLE[v >> 26 & 0x3f]);
            sb.push_str(TABLE[v >> 20 & 0x3f]);
            sb.push_str(TABLE[v >> 14 & 0x3f]);
            sb.push_str(TABLE[v >> 8 & 0x3f]);
            i += 3;
        }
    }
    sb
}

#[inline(always)]
unsafe fn decode_last_chunk(rt: *const u32, ptr: *const u32, offset: usize) -> (u32, usize) {
    let c = ptr.add(offset).read() as usize;
    (
        rt.add(c >> 24 & 0xff).read()
            | rt.add(c >> 16 & 0xff).read() << 6
            | rt.add(c >> 8 & 0xff).read() << 12
            | rt.add(c & 0xff).read() << 18,
        if c >> 16 & 0xff == 61 {
            1
        } else if c >> 24 & 0xff == 61 {
            2
        } else {
            3
        },
    )
}

/// Decodes Base64, uses `data` as a buffer for in-place decoding.
///
/// Returns a slice of `data` with decoded data.
pub fn decode(data: &mut [u8]) -> &[u8] {
    let data_ptr = data.as_ptr() as *const u16;
    let out_ptr = data.as_mut_ptr() as *mut u8;

    if data.len() < 4 {
        return &[];
    }

    let max_size = (data.len() - 4) >> 2;
    let last_chunk_pos = max_size * 3;
    let mut out_i = 0;
    for i in (0..max_size << 1).step_by(2) {
        let value = unsafe {
            *RPT_PTR.add(*data_ptr.add(i) as usize) << 12 | *RPT_PTR.add(*data_ptr.add(i + 1) as usize)
        };
        unsafe { (out_ptr.add(out_i) as *mut u32).write_unaligned((value << 8).swap_bytes()) }
        out_i += 3;
    }

    let (last_chunk, last_chunk_len) = unsafe { decode_last_chunk(RT_PTR, data_ptr as *const u32, max_size) };
    unsafe { (out_ptr.add(last_chunk_pos) as *mut u32).write_unaligned((last_chunk << 8).swap_bytes()) }

    &data[..last_chunk_pos + last_chunk_len]
}

#[cfg(test)]
mod tests {
    use crate as faster_base64;

    use std::fs::File;
    use std::io::Read;
    use std::time::Instant;

    #[test]
    fn encode() {
        assert_eq!(faster_base64::encode(b"Hello"), "SGVsbG8=");
    }

    #[test]
    fn encode_chunk_equal() {
        assert_eq!(faster_base64::encode(b"Hel"), "SGVs");
    }

    #[test]
    fn encode_chunk_long() {
        assert_eq!(
            faster_base64::encode(b"HelHelHelHelHelHelHelHel"),
            "SGVsSGVsSGVsSGVsSGVsSGVsSGVsSGVs"
        );
    }

    #[test]
    fn encode_chunk_remain_1() {
        assert_eq!(faster_base64::encode(b"Hell"), "SGVsbA==");
    }

    #[test]
    fn decode() {
        assert_eq!(
            faster_base64::decode(&mut "SGVsbG8=".as_bytes().to_vec()),
            b"Hello"
        );
    }

    #[test]
    fn decode_chunk_equal() {
        assert_eq!(
            faster_base64::decode(&mut "SGVs".as_bytes().to_vec()),
            b"Hel"
        );
    }

    #[test]
    fn decode_chunk_long() {
        assert_eq!(
            faster_base64::decode(&mut "SGVsSGVsSGVsSGVsSGVsSGVsSGVsSGVs".as_bytes().to_vec()),
            b"HelHelHelHelHelHelHelHel"
        );
    }

    #[test]
    fn decode_chunk_remain_1() {
        assert_eq!(
            faster_base64::decode(&mut "SGVsbA==".as_bytes().to_vec()),
            b"Hell"
        );
    }

    #[test]
    fn benchmark() {
        const MB_SIZE: usize = 1024;
        let buf_size = MB_SIZE * 1024 * 1024;

        println!("Allocating buffer...");
        let mut buf = vec![0u8; buf_size];

        println!("Reading {} MB of test data from /dev/urandom...", MB_SIZE);
        let mut f = File::open("/dev/urandom").unwrap();
        f.read_exact(&mut buf).unwrap();

        println!("Starting encode benchmark...");
        let start = Instant::now();
        let encoded = faster_base64::encode(&buf);
        let encode_duration = start.elapsed();
        println!("Encoded {} MB in {:?}", MB_SIZE, encode_duration);
        println!("Starting decode benchmark...");
        let mut bind = encoded.as_bytes().to_vec();
        let start = Instant::now();
        let decoded = faster_base64::decode(&mut bind);
        let decode_duration = start.elapsed();
        println!("Decoded {} MB in {:?}", MB_SIZE, decode_duration);

        assert_eq!(decoded, buf);
    }
}
