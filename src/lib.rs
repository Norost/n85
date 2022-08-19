#![doc = include_str!("../README.md")]
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code, missing_docs)]
#![feature(slice_as_chunks)]

/// Decode a N85 string.
///
/// Note that even if decoding fails, the contents in `output` may be overwritten.
pub fn decode(input: &[u8], output: &mut [u8]) -> Result<usize, DecodeError> {
	if input.len() % 5 == 1 {
		return Err(DecodeError::InvalidLength);
	}

	let out_len = input.len() * 4 / 5;
	if output.len() < out_len {
		return Err(DecodeError::OutputTooShort);
	}

	let (inp, in_rem) = input.as_chunks::<5>();
	let (out, out_rem) = output[..out_len].as_chunks_mut::<4>();

	if inp.len() != out.len() {
		return Err(DecodeError::OutputTooShort);
	}

	for c in input {
		if !(b'('..=b'[').contains(c) && !(b']'..=b'}').contains(c) {
			return Err(DecodeError::InvalidChar);
		}
	}

	for (a, out) in inp.iter().zip(out.iter_mut()) {
		let mut v = 0;
		for &c in a.iter().rev() {
			v *= 85;
			v += u32::from(dec(c));
		}
		out.copy_from_slice(&v.to_le_bytes());
	}

	match (in_rem, out_rem) {
		(&[], []) => {}
		(&[x, y], [a]) => {
			*a = dec(x) + dec(y) * 85;
		}
		(&[x, y, z], [a, b]) => {
			let f = |n| u16::from(dec(n));
			let v = f(x) + f(y) * 85 + f(z) * 85 * 85;
			[*a, *b] = v.to_le_bytes();
		}
		(&[x, y, z, w], [a, b, c]) => {
			let f = |n| u32::from(dec(n));
			let v = f(x) + f(y) * 85 + f(z) * 85 * 85 + f(w) * 85 * 85 * 85;
			[*a, *b, *c, _] = v.to_le_bytes();
		}
		_ => return Err(DecodeError::OutputTooShort),
	}

	Ok(out_len)
}

/// Error returned if decoding failed.
#[derive(Clone, Copy, Debug)]
pub enum DecodeError {
	/// An invalid character was encountered.
	InvalidChar,
	/// The length is invalid, i.e. `input.len() % 5 == 1`, which is not possible.
	InvalidLength,
	/// The output buffer is too small to store the decoded data.
	OutputTooShort,
}

/// Encode an arbitrary byte string.
pub fn encode(input: &[u8], output: &mut [u8]) -> Result<usize, EncodeError> {
	let out_len = input.len() * 5 / 4 + usize::from(input.len() % 4 >= 1);
	if output.len() < out_len {
		return Err(EncodeError::OutputTooShort);
	}

	let (inp, in_rem) = input.as_chunks::<4>();
	let (out, out_rem) = output[..out_len].as_chunks_mut::<5>();

	if out.len() != inp.len() {
		return Err(EncodeError::OutputTooShort);
	}
	for (mut v, out) in inp
		.iter()
		.map(|a| u32::from_le_bytes(*a))
		.zip(out.iter_mut())
	{
		for c in out.iter_mut() {
			*c = enc((v % 85) as u8);
			v /= 85;
		}
	}

	match (in_rem, out_rem) {
		(&[], []) => {}
		(&[a], [x, y]) => {
			*x = enc(a % 85);
			*y = enc(a / 85);
		}
		(&[a, b], [x, y, z]) => {
			let v = u16::from_le_bytes([a, b]);
			*x = enc((v % 85) as u8);
			*y = enc((v / 85 % 85) as u8);
			*z = enc((v / (85 * 85)) as u8);
		}
		(&[a, b, c], [x, y, z, w]) => {
			let v = u32::from_le_bytes([a, b, c, 0]);
			*x = enc((v % 85) as u8);
			*y = enc((v / 85 % 85) as u8);
			*z = enc((v / (85 * 85) % 85) as u8);
			*w = enc((v / (85 * 85 * 85)) as u8);
		}
		_ => return Err(EncodeError::OutputTooShort),
	}

	Ok(out_len)
}

/// Error returned if encoding fails.
#[derive(Clone, Copy, Debug)]
pub enum EncodeError {
	/// The output buffer is too small to store the encoded string.
	OutputTooShort,
}

fn dec(n: u8) -> u8 {
	n - u8::from(n >= b'\\') - b'('
}

fn enc(n: u8) -> u8 {
	let n = n + b'(';
	n + u8::from(n >= b'\\')
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn enc_dec_12() {
		let s = b"Abracadabra!";
		let mut a = [0; 15];
		let l = encode(s, &mut a).unwrap();
		assert_eq!(l, 15);

		let mut b = [0; 12];
		let l = decode(&a[..l], &mut b).unwrap();
		assert_eq!(&b[..l], s);
	}

	#[test]
	fn enc_dec_13() {
		let s = b"Abracadabra!1";
		let mut a = [0; 17];
		let l = encode(s, &mut a).unwrap();
		assert_eq!(l, 17);

		let mut b = [0; 13];
		let l = decode(&a[..l], &mut b).unwrap();
		assert_eq!(&b[..l], s);
	}

	#[test]
	fn enc_dec_14() {
		let s = b"Abracadabra!12";
		let mut a = [0; 18];
		let l = encode(s, &mut a).unwrap();
		assert_eq!(l, 18);

		let mut b = [0; 14];
		let l = decode(&a[..l], &mut b).unwrap();
		assert_eq!(&b[..l], s);
	}

	#[test]
	fn enc_dec_15() {
		let s = b"Abracadabra!123";
		let mut a = [0; 19];
		let l = encode(s, &mut a).unwrap();
		assert_eq!(l, 19);

		let mut b = [0; 15];
		let l = decode(&a[..l], &mut b).unwrap();
		assert_eq!(&b[..l], s);
	}

	#[test]
	fn enc_dec_64() {
		let s = &[0; 64];
		let mut a = [0; 80];
		let l = encode(s, &mut a).unwrap();
		assert_eq!(l, 80);

		let mut b = [0; 64];
		let l = decode(&a[..l], &mut b).unwrap();
		assert_eq!(&b[..l], s);
	}

	#[test]
	fn enc_max() {
		let s = &[84];
		let mut a = [0; 2];
		let l = encode(s, &mut a).unwrap();
		assert_eq!(l, 2);
		assert_eq!(a, [b'}', b'(']);

		let s = &[85];
		let mut a = [0; 2];
		let l = encode(s, &mut a).unwrap();
		assert_eq!(l, 2);
		assert_eq!(a, [b'(', b')']);
	}
}
