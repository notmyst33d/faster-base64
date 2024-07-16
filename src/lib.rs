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
    let data_ptr = data.as_ptr() as *const u32;
    let out_ptr = data.as_mut_ptr() as *mut u8;
    let rt = REVERSE_TABLE.as_ptr();
    let rpt = REVERSE_PAIR_TABLE.as_ptr();

    if data.len() < 4 {
        return &[];
    }

    let max_size = (data.len() - 4) >> 2;
    let last_chunk_pos = max_size * 3;
    for (i, out_i) in (0..max_size).zip((0..last_chunk_pos).step_by(3)) {
        let dp = unsafe { *data_ptr.add(i) };
        let value = unsafe {
            *rpt.add((dp & 0xffff) as usize) << 12 | *rpt.add((dp >> 16 & 0xffff) as usize)
        };
        unsafe { *(out_ptr.add(out_i) as *mut u32) = (value << 8).swap_bytes() }
    }

    let (last_chunk, last_chunk_len) = unsafe { decode_last_chunk(rt, data_ptr, max_size) };
    unsafe { *(out_ptr.add(last_chunk_pos) as *mut u32) = (last_chunk << 8).swap_bytes() }

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
        let mut buf = vec![0u8; MB_SIZE * 1024 * 1024];
        println!("Read test data");
        let mut f = File::open("/dev/urandom").unwrap();
        f.read(&mut buf).unwrap();
        println!("Start encode benchmark");
        let mut start = Instant::now();
        let data2 = faster_base64::encode(&buf);
        let mut data = data2.as_bytes().to_vec();
        println!("Encoded {} MB in {:?}", MB_SIZE, start.elapsed());
        println!("Start decode benchmark");
        start = Instant::now();
        faster_base64::decode(&mut data);
        println!("Decoded {} MB in {:?}", MB_SIZE, start.elapsed());
    }
}
