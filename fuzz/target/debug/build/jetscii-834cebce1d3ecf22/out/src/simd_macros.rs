
#[allow(unused_macros)]
macro_rules! simd_bytes {
($b00:expr) => ($crate::simd::Bytes::new([$b00, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 1));
($b00:expr, $b01:expr) => ($crate::simd::Bytes::new([$b00, $b01, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 2));
($b00:expr, $b01:expr, $b02:expr) => ($crate::simd::Bytes::new([$b00, $b01, $b02, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 3));
($b00:expr, $b01:expr, $b02:expr, $b03:expr) => ($crate::simd::Bytes::new([$b00, $b01, $b02, $b03, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 4));
($b00:expr, $b01:expr, $b02:expr, $b03:expr, $b04:expr) => ($crate::simd::Bytes::new([$b00, $b01, $b02, $b03, $b04, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 5));
($b00:expr, $b01:expr, $b02:expr, $b03:expr, $b04:expr, $b05:expr) => ($crate::simd::Bytes::new([$b00, $b01, $b02, $b03, $b04, $b05, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 6));
($b00:expr, $b01:expr, $b02:expr, $b03:expr, $b04:expr, $b05:expr, $b06:expr) => ($crate::simd::Bytes::new([$b00, $b01, $b02, $b03, $b04, $b05, $b06, 0, 0, 0, 0, 0, 0, 0, 0, 0], 7));
($b00:expr, $b01:expr, $b02:expr, $b03:expr, $b04:expr, $b05:expr, $b06:expr, $b07:expr) => ($crate::simd::Bytes::new([$b00, $b01, $b02, $b03, $b04, $b05, $b06, $b07, 0, 0, 0, 0, 0, 0, 0, 0], 8));
($b00:expr, $b01:expr, $b02:expr, $b03:expr, $b04:expr, $b05:expr, $b06:expr, $b07:expr, $b08:expr) => ($crate::simd::Bytes::new([$b00, $b01, $b02, $b03, $b04, $b05, $b06, $b07, $b08, 0, 0, 0, 0, 0, 0, 0], 9));
($b00:expr, $b01:expr, $b02:expr, $b03:expr, $b04:expr, $b05:expr, $b06:expr, $b07:expr, $b08:expr, $b09:expr) => ($crate::simd::Bytes::new([$b00, $b01, $b02, $b03, $b04, $b05, $b06, $b07, $b08, $b09, 0, 0, 0, 0, 0, 0], 10));
($b00:expr, $b01:expr, $b02:expr, $b03:expr, $b04:expr, $b05:expr, $b06:expr, $b07:expr, $b08:expr, $b09:expr, $b10:expr) => ($crate::simd::Bytes::new([$b00, $b01, $b02, $b03, $b04, $b05, $b06, $b07, $b08, $b09, $b10, 0, 0, 0, 0, 0], 11));
($b00:expr, $b01:expr, $b02:expr, $b03:expr, $b04:expr, $b05:expr, $b06:expr, $b07:expr, $b08:expr, $b09:expr, $b10:expr, $b11:expr) => ($crate::simd::Bytes::new([$b00, $b01, $b02, $b03, $b04, $b05, $b06, $b07, $b08, $b09, $b10, $b11, 0, 0, 0, 0], 12));
($b00:expr, $b01:expr, $b02:expr, $b03:expr, $b04:expr, $b05:expr, $b06:expr, $b07:expr, $b08:expr, $b09:expr, $b10:expr, $b11:expr, $b12:expr) => ($crate::simd::Bytes::new([$b00, $b01, $b02, $b03, $b04, $b05, $b06, $b07, $b08, $b09, $b10, $b11, $b12, 0, 0, 0], 13));
($b00:expr, $b01:expr, $b02:expr, $b03:expr, $b04:expr, $b05:expr, $b06:expr, $b07:expr, $b08:expr, $b09:expr, $b10:expr, $b11:expr, $b12:expr, $b13:expr) => ($crate::simd::Bytes::new([$b00, $b01, $b02, $b03, $b04, $b05, $b06, $b07, $b08, $b09, $b10, $b11, $b12, $b13, 0, 0], 14));
($b00:expr, $b01:expr, $b02:expr, $b03:expr, $b04:expr, $b05:expr, $b06:expr, $b07:expr, $b08:expr, $b09:expr, $b10:expr, $b11:expr, $b12:expr, $b13:expr, $b14:expr) => ($crate::simd::Bytes::new([$b00, $b01, $b02, $b03, $b04, $b05, $b06, $b07, $b08, $b09, $b10, $b11, $b12, $b13, $b14, 0], 15));
($b00:expr, $b01:expr, $b02:expr, $b03:expr, $b04:expr, $b05:expr, $b06:expr, $b07:expr, $b08:expr, $b09:expr, $b10:expr, $b11:expr, $b12:expr, $b13:expr, $b14:expr, $b15:expr) => ($crate::simd::Bytes::new([$b00, $b01, $b02, $b03, $b04, $b05, $b06, $b07, $b08, $b09, $b10, $b11, $b12, $b13, $b14, $b15], 16));
}
