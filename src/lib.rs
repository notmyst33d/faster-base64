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

#[inline]
unsafe fn decode_chunk_no_padding(rpt: *const u32, ptr: *const u8, offset: usize) -> u32 {
    let dp = (ptr.add(offset) as *const u32).read() as usize;
    rpt.add(dp & 0xffff).read() << 12 | rpt.add(dp >> 16 & 0xffff).read()
}

#[inline]
unsafe fn decode_chunk(rt: *const u32, ptr: *const u8, offset: usize) -> (u32, usize) {
    let c = (ptr.add(offset) as *const u32).read() as usize;
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

pub fn decode(data: &str) -> Vec<u8> {
    let len = data.len();
    let data_ptr = data.as_ptr();
    let mut out = Vec::with_capacity(len);
    let out_ptr: *const u8 = out.as_ptr();
    let mut out_i = 0;
    let rt = REVERSE_TABLE.as_ptr();
    let rpt = REVERSE_PAIR_TABLE.as_ptr();

    if len < 4 {
        return vec![];
    }

    let mut i = 0;
    while i < len - 4 {
        let value = unsafe { decode_chunk_no_padding(rpt, data_ptr, i) };
        unsafe { (out_ptr.add(out_i) as *mut u32).write((value << 8).swap_bytes()) }
        out_i += 3;
        i += 4;
    }

    let (lc, lc_len) = unsafe { decode_chunk(rt, data_ptr, len - 4) };
    unsafe { (out_ptr.add(out_i) as *mut u32).write((lc << 8).swap_bytes()) }
    out_i += lc_len;

    unsafe {
        out.set_len(out_i);
    }
    out
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
    fn encode_chunk_remain_1() {
        assert_eq!(faster_base64::encode(b"Hell"), "SGVsbA==");
    }

    #[test]
    fn decode() {
        assert_eq!(faster_base64::decode("SGVsbG8="), b"Hello".to_vec());
    }

    #[test]
    fn decode_chunk_equal() {
        assert_eq!(faster_base64::decode("SGVs"), b"Hel".to_vec());
    }

    #[test]
    fn decode_chunk_remain_1() {
        assert_eq!(faster_base64::decode("SGVsbA=="), b"Hell".to_vec());
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
        let data = faster_base64::encode(&buf);
        println!("Encoded {} MB in {:?}", MB_SIZE, start.elapsed());
        println!("Start decode benchmark");
        start = Instant::now();
        faster_base64::decode(&data);
        println!("Decoded {} MB in {:?}", MB_SIZE, start.elapsed());
    }
}
