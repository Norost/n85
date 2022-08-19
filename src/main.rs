#![feature(byte_slice_trim_ascii)]

use std::io::{Read, Write};

const N: usize = 1 << 13;

fn main() -> Result<(), &'static str> {
	match std::env::args().nth(1).as_deref() {
		None => encode(),
		Some("-d") => decode(),
		_ => Err("invalid option"),
	}
}

fn encode() -> Result<(), &'static str> {
	let mut rd = std::io::stdin().lock();
	let mut wr = std::io::stdout().lock();
	let mut buf = [0; N];
	let mut out = [0; N * 5 / 4];
	loop {
		let mut n = 0;
		let mut l = rd.read(&mut buf).unwrap();
		if l == 0 {
			break Ok(());
		}
		n += l;
		while n % 4 != 0 {
			l = rd.read(&mut buf[n..][..4 - n % 4]).unwrap();
			if l == 0 {
				break;
			}
			n += l;
		}
		let e = n85::encode(&buf[..n], &mut out).unwrap();
		wr.write_all(&out[..e]).unwrap();
		if l == 0 {
			break Ok(());
		}
	}
}

fn decode() -> Result<(), &'static str> {
	let mut rd = std::io::stdin().lock();
	let mut wr = std::io::stdout().lock();
	let mut buf = [0; N * 5 / 4];
	let mut out = [0; N];
	loop {
		let mut n = 0;
		let mut l = rd.read(&mut buf).unwrap();
		if l == 0 {
			break Ok(());
		}
		n += l;
		while n % 5 != 0 {
			l = rd.read(&mut buf[n..][..5 - n % 5]).unwrap();
			if l == 0 {
				break;
			}
			n += l;
		}
		// Trim since there's often a newline or some other whitespace.
		let buf = buf[..n].trim_ascii();
		let d = n85::decode(buf, &mut out).unwrap();
		wr.write_all(&out[..d]).unwrap();
		if l == 0 {
			break Ok(());
		}
	}
}
