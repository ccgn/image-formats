use std::io;
use std::cmp;
use std::slice;
use std::io::IoResult;

static LITERALLENGTHCODES: u16 = 286;
static DISTANCECODES: u16 = 30;
static CODEORDER: [u8, ..19] = [
	16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15
];
static LENGTHS: [u16, ..29] = [
	 3,  4,  5,   6,   7,   8,   9,  10,  11, 13, 
	15, 17, 19,  23,  27,  31,  35,  43,  51, 59, 
	67, 83, 99, 115, 131, 163, 195, 227, 258
];
static EXTRA_LENGTHS: [u8, ..29] = [
	0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 
	1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 
	4, 4, 4, 4, 5, 5, 5, 5, 0
];
static DISTANCES: [u16, ..30] = [
	   1,    2,      3,    4,    5,    7,    9,    13,    17,    25, 
	  33,   49,     65,   97,  129,  193,  257,   385,   513,   769, 
	1025,  1537,  2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577
];
static EXTRA_DISTANCES: [u8, ..30] = [
	0, 0,  0,  0,  1,  1,  2,  2,  3,  3, 
	4, 4,  5,  5,  6,  6,  7,  7,  8,  8, 
	9, 9, 10, 10, 11, 11, 12, 12, 13, 13
];

static TABLESIZE: u8 = 9; 

#[deriving(Eq, Clone)]
enum TableElement {
	Symbol(u16, u8),
	Table(u16, ~[TableElement]),
	Nothing
}

impl TableElement {
	pub fn put(&mut self, index: u16, elem: TableElement) {
		match *self {
			Table(_, ref mut a) => a[index as uint] = elem,
			_				=> fail!("requires Table()"),
		}
	}
}

enum BlockType {Stored, Compressed}

pub struct Inflater<R> {
	h: HuffReader<R>,

	buf: ~[u8],
	pos: u64,

	final: bool,
	btype: BlockType,
	block_length: u32,
	
	ctable: ~[TableElement],
	lltable: ~[TableElement],
	dtable: ~[TableElement],
}

impl<R: Reader> Inflater<R> {
	pub fn new(r: R) -> Inflater<R> {
		Inflater {
			h: HuffReader::new(r),
			
			buf: ~[],
			pos: 0,
			
			final: false,
			block_length: 0,
			btype: Stored,
			
			ctable: ~[],
			lltable: ~[],
			dtable: ~[],
		}
	}

	pub fn eof(&self) -> bool {
		self.final && (self.pos as uint == self.buf.len())
	}

	pub fn inner<'a>(&'a mut self) -> &'a mut R {
		&mut self.h.r
	}

	fn read_block_type(&mut self) -> IoResult<()> {
		let final = try!(self.h.receive(1));
		self.final = final == 1;

		let bits = try!(self.h.receive(2));
		match bits {
			0b00 => {
				let _ = try!(self.read_stored_block_length());
				self.btype = Stored;
			}
			0b01 => {
				self.create_fixed_tables();
				self.btype = Compressed;
			}
			0b10 => {
				let _ = try!(self.read_dynamic_tables());
				self.btype = Compressed;
			}
			_ => fail!("reserved block type")
		}

		Ok(())
	}

	fn read_dynamic_tables(&mut self) -> IoResult<()> {
		let totalcodes = LITERALLENGTHCODES + DISTANCECODES;
		
		let hlit  = try!(self.h.receive(5)) + 257;
		let hdist = try!(self.h.receive(5)) + 1;
		let hclen = try!(self.h.receive(4)) + 4;

		let mut code_lengths = slice::from_elem(CODEORDER.len(), 0u8);
		
		for i in range(0, hclen) {
			let length = try!(self.h.receive(3));
			code_lengths[CODEORDER[i]] = length as u8;
		}

		self.ctable = table_from_lengths(code_lengths);
		let mut all_lengths = slice::from_elem(totalcodes as uint, 0u8);

		let mut i = 0;
		while i < hlit + hdist {
			let s = try!(self.h.decode_symbol(self.ctable));

			match s {
				0 .. 15 => {
					all_lengths[i] = s as u8;
					i += 1;
				}
				16 => {
					let repeat = 3 + try!(self.h.receive(2));
					
					for _ in range(0, repeat) {
						all_lengths[i] = all_lengths[i - 1];
						i += 1; 
					}
				}
				17 => i += 3 + try!(self.h.receive(3)),
				18 => i += 11 + try!(self.h.receive(7)),
				_ => fail!("out of range code length code symbol")
			}
		}

		let ll_lengths = all_lengths.slice_to(hlit as uint);
		let d_lengths  = all_lengths.slice_from(hlit as uint);

		self.lltable = table_from_lengths(ll_lengths);
		self.dtable = table_from_lengths(d_lengths);

		Ok(())
	}

	fn create_fixed_tables(&mut self) {
		let lengths = slice::from_fn(288, |i|
			if i < 144 { 8u8 }
			else if i < 256 { 9u8 }
			else if i < 280 { 7u8 }
			else { 8u8 }
		);
		self.lltable = table_from_lengths(lengths);

		let lengths = slice::from_elem(DISTANCECODES as uint, 5u8);
		self.dtable = table_from_lengths(lengths);
	}

	fn read_stored_block_length(&mut self) -> IoResult<()> {
		self.h.byte_align();

		let len   = try!(self.h.receive(16));
		let _nlen = try!(self.h.receive(16));

		self.block_length = len as u32;

		Ok(())
	}

	fn read_stored_block(&mut self) -> IoResult<()> {
		for _ in range(0, self.block_length) {
			let a = try!(self.h.receive(8));
			
			self.buf.push(a as u8);
			self.h.consume(8);
		}

		self.block_length = 0;
		Ok(()) 
	}

	fn read_compressed_block(&mut self) -> IoResult<()> {
		loop {
			let s = try!(self.h.decode_symbol(self.lltable));

			match s {
				literal @ 0 .. 255 => self.buf.push(literal as u8),
				256 => break,
				length @ 257 .. 285 => {
					let length = length - 257;
					
					let bits = EXTRA_LENGTHS[length];
					let extra = try!(self.h.receive(bits));
					
					let length = LENGTHS[length] + extra;

					let distance = try!(self.h.decode_symbol(self.dtable));

					let bits = EXTRA_DISTANCES[distance];
					let extra = try!(self.h.receive(bits));
					
					let distance = DISTANCES[distance] + extra;

					let len = self.buf.len();
					for i in range(0, length) {
						let s = self.buf[len - distance as uint + i as uint];
						self.buf.push(s);
					}
				}

				_ => fail!("out of range symbol")
			}
		}

		Ok(())
	}
}

impl<R: Reader> Reader for Inflater<R> {
	fn read(&mut self, buf: &mut [u8]) -> IoResult<uint> {
		if self.pos as uint == self.buf.len() {
			if self.final {
				return Err(io::standard_error(io::EndOfFile))
			}

			let _ = try!(self.read_block_type());
			let _ = match self.btype {
				Stored => try!(self.read_stored_block()),
				Compressed => try!(self.read_compressed_block()) 
			};
		}

		let n = cmp::min(buf.len(), self.buf.len() - self.pos as uint);
		for i in range(0, n) {
			buf[i] = self.buf[self.pos as uint + i];
		}

		self.pos += n as u64;
		Ok(n)
	}
}

fn reverse(a: u16) -> u16 {
	let b = (((!0x5555) & a) >> 1) | ((0x5555 & a) << 1);
	let c = (((!0x3333) & b) >> 2) | ((0x3333 & b) << 2);
	let d = (((!0x0F0F) & c) >> 4) | ((0x0F0F & c) << 4);
 	
 	(((!0x00FF) & d) >> 8) | ((0x00FF & d) << 8)
}

fn table_from_lengths(lengths: &[u8]) -> ~[TableElement] {
	let mut max_len = 0;
	let mut code = 0u16;
	let mut next_code = slice::from_elem(16, 0u16);
	let mut bl_count = slice::from_elem(16, 0u8);

	for &len in lengths.iter() {
		bl_count[len] += 1;

		if len > max_len {
			max_len = len;
		}
	}
	
	let max_overflow = max_len - TABLESIZE;
	bl_count[0] = 0;

	for bits in range(1, 16) {
		code = (code + bl_count[bits - 1] as u16) << 1;
		next_code[bits] = code;
	}

	let mut lut = slice::from_elem(1 << TABLESIZE, Nothing);

	for (i, &len) in lengths.iter().enumerate() {
		if len == 0 {
			continue
		}

		let code = next_code[len];
		let code = reverse(code) >> (16 - len);				
		
		if len <= TABLESIZE {
			let r = TABLESIZE - len;

			for j in range(0u16, 1 << r) {
				let index = (j << len as u16) + code;
				lut[index as uint] = Symbol(i as u16, len);
			}
		}
		else {
			let index = code & ((1 << TABLESIZE) - 1);
			
			if lut[index as uint] == Nothing {
				let mask  = (1 << max_overflow) - 1;
				let array = slice::from_elem(1 << max_overflow, Nothing);
				
				lut[index as uint] = Table(mask, array); 
			}

			let code = code >> TABLESIZE;
			let r = max_len - len;

			for j in range(0u16, 1 << r) {
				let k = (j << (len - TABLESIZE)) + code;
				let s = Symbol(i as u16, len - TABLESIZE);

				lut[index as uint].put(k, s);
			}
		}

		next_code[len] += 1;
	}

	lut
}

struct HuffReader<R> {
	pub r: R,

	bits: u32,
	num_bits: u8,
}

impl<R: Reader> HuffReader<R> {
	pub fn new(r: R) -> HuffReader<R> {
		HuffReader {r: r, bits: 0, num_bits: 0}
	}

	pub fn guarantee(&mut self, n: u8) -> IoResult<()> {
		while self.num_bits < n as u8 {
			let byte = try!(self.r.read_u8());

			self.bits |= byte as u32 << self.num_bits;
			self.num_bits += 8;
		}

		Ok(())
	}

	pub fn byte_align(&mut self) {
		let n = self.bits & 0b111;
		
		self.bits >>= n;
		self.num_bits -= n as u8;
	}

	pub fn consume(&mut self, n: u8) {
		self.bits >>= n as u32;
		self.num_bits -= n;
	}

	pub fn receive(&mut self, n: u8) -> IoResult<u16> {
		let _ = try!(self.guarantee(n));

		let val = self.bits & ((1 << n) - 1);
		self.consume(n);

		Ok(val as u16)
	}

	pub fn decode_symbol(&mut self, table: &[TableElement]) -> IoResult<u16> {
		let _ = try!(self.guarantee(1));

		loop {
			let index = self.bits & ((1 << TABLESIZE) - 1);
			
			let (val, size) = match table[index] {
				Symbol(val, size) => (val, size),

				Table(mask, ref a) => {
					let index = (self.bits >> TABLESIZE) & mask as u32;
					
					match a[index] {
						Symbol(val, size) => (val, size + TABLESIZE),
						_ 				  => fail!("bad huffman code")
					}
				}

				Nothing => fail!("bad huffman code")
			};
			
			if size <= self.num_bits {
				self.consume(size);
				return	Ok(val)
			}

			let _ = try!(self.guarantee(size));
		}
	}
}