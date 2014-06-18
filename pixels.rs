//! Types and functions for dealing with pixels.

use std::num::cast;
use std::num::Bounded;

use sample;
use colortype;
use colortype::ColorType;

///A type to hold a grayscale pixel
#[packed]
#[deriving(Default, PartialEq, Clone, Show, Copy)]
pub struct Luma<T>(pub T);

impl<T: Primitive> Luma<T> {
	/// Returns the channels of this pixel as a tuple
	pub fn channel(&self) -> T {
		match *self {
			Luma(l) => l
		}
	}
}

/// A type to hold a grayscale pixel with an alha channel
#[packed]
#[deriving(Default, PartialEq, Clone, Show, Copy)]
pub struct LumaA<T>(pub T, pub T);

impl<T: Primitive> LumaA<T> {
	/// Returns the channels of this pixel as a tuple
	pub fn channels(&self) -> (T, T) {
		match *self {
			LumaA(l, a) => (l, a)
		}
	}

	/// Returns the alpha channel of this pixel
	pub fn alpha(&self) -> T {
		match *self {
			LumaA(_, a) => a
		}
	}
}

/// A type to hold an RGB pixel
#[packed]
#[deriving(Default, PartialEq, Clone, Show, Copy)]
pub struct Rgb<T>(pub T, pub T, pub T);

impl<T: Primitive> Rgb<T> {
	/// Returns the channels of this pixel as a tuple
	pub fn channels(&self) -> (T, T, T) {
		match *self {
			Rgb(r, g, b) => (r, g, b)
		}
	}
}

/// A type to hold an RGB pixel with an alpha channel
#[packed]
#[deriving(Default, PartialEq, Clone, Show, Copy)]
pub struct Rgba<T>(pub T, pub T, pub T, pub T);

impl<T: Primitive> Rgba<T> {
	/// Returns the channels of this pixel as a tuple
	pub fn channels(&self) -> (T, T, T, T) {
		match *self {
			Rgba(r, g, b, a) => (r, g, b, a)
		}
	}

	/// Returns the alpha channel of this pixel
	pub fn alpha(&self) -> T {
		match *self {
			Rgba(_, _, _, a) => a
		}
	}
}

/// A trait that all pixels implement.
pub trait Pixel<T> {
	fn from_channels(&self, a: T, b: T, c: T, d: T) -> Self;
	/// Convert this pixel to RGB
	fn to_rgb(&self) -> Rgb<T>;

	/// Convert this pixel to RGB with an alpha channel
	fn to_rgba(&self) -> Rgba<T>;

	/// Convert this pixel to luma
	fn to_luma(&self) -> Luma<T>;

	/// Convert this pixel to luma with an alpha channel
	fn to_luma_alpha(&self) -> LumaA<T>;

	/// Invert the color of this pixel
	fn invert(&mut self);

	/// Apply the function ```f``` to each channel of this pixel.
	/// If there is an alpha channel it is not changed.
	fn map(&self, f: |T| -> T) -> Self;

	/// Apply the function ```f``` to each channel of this pixel and
	/// ```other``` pairwise.
	fn map2(&self, other: Self, f: |T, T| -> T) -> Self;

	/// Returns the channels of this pixes as a 4 tuple. If the pixel
	/// has less than 4 channels the remainder is filled with the maximum value
	fn channels4(&self) -> (T, T, T, T);
}

impl<T: Primitive> Pixel<T> for Rgb<T> {
	fn from_channels(&self, a: T, b: T, c: T, _: T) -> Rgb<T> {
		Rgb(a, b, c)
	}

	fn to_luma(&self) -> Luma<T> {
		let (r, g, b) = self.channels();

		let l = 0.2125f32 * r.to_f32().unwrap() +
			0.7154f32 * g.to_f32().unwrap() +
			0.0721f32 * b.to_f32().unwrap();

		Luma(cast::<f32, T>(l).unwrap())
	}

	fn to_luma_alpha(&self) -> LumaA<T> {
		let l = self.to_luma().channel();

		LumaA(l, Bounded::max_value())
	}

	fn to_rgb(&self) -> Rgb<T> {
		self.clone()
	}

	fn to_rgba(&self) -> Rgba<T> {
		let (r, g, b) = self.channels();

		Rgba(r, g, b, Bounded::max_value())
	}

	fn invert(&mut self) {
		let (r, g, b) = self.channels();

		let max: T = Bounded::max_value();

		let r1 = max - r;
		let g1 = max - g;
		let b1 = max - b;

		*self = Rgb(r1, g1, b1)
	}

	fn map(&self, f: |a: T| -> T) -> Rgb<T> {
		let (r, g, b) = self.channels();

		let r1 = f(r);
		let g1 = f(g);
		let b1 = f(b);

		Rgb(r1, g1, b1)
	}

	fn map2(&self, other: Rgb<T>, f: |a: T, b: T| -> T) -> Rgb<T> {
		let (r1, g1, b1) = self.channels();
		let (r2, g2, b2) = other.channels();

		let r3 = f(r1, r2);
		let g3 = f(g1, g2);
		let b3 = f(b1, b2);

		Rgb(r3, g3, b3)
	}

	fn channels4(&self) ->(T, T, T, T) {
		let (r, g, b) = self.channels();

		(r, g, b, Bounded::max_value())
	}
}

impl<T: Primitive> Pixel<T> for Rgba<T> {
	fn from_channels(&self, a: T, b: T, c: T, d: T) -> Rgba<T> {
		Rgba(a, b, c, d)
	}

	fn to_luma(&self) -> Luma<T> {
		self.to_rgb().to_luma()
	}

	fn to_luma_alpha(&self) -> LumaA<T> {
		let l = self.to_luma().channel();
		let a = self.alpha();

		LumaA(l, a)
	}

	fn to_rgb(&self) -> Rgb<T> {
		let (r, g, b, _) = self.channels();

		Rgb(r, g, b)
	}

	fn to_rgba(&self) -> Rgba<T> {
		self.clone()
	}

	fn invert(&mut self) {
		let (r, g, b) = self.to_rgb().channels();
		let a = self.alpha();

		let max: T = Bounded::max_value();

		*self = Rgba(max - r, max - g, max - b, a)
	}

	fn map(&self, f: |a: T| -> T) -> Rgba<T> {
		let (r, g, b, a) = self.channels();

		let r1 = f(r);
		let g1 = f(g);
		let b1 = f(b);

		Rgba(r1, g1, b1, a)
	}

	fn map2(&self, other: Rgba<T>, f: |a: T, b: T| -> T) -> Rgba<T> {
		let (r1, g1, b1, a1) = self.channels();
		let (r2, g2, b2, _) = other.channels();

		let r3 = f(r1, r2);
		let g3 = f(g1, g2);
		let b3 = f(b1, b2);

		Rgba(r3, g3, b3, a1)
	}

	fn channels4(&self) ->(T, T, T, T) {
		let (r, g, b, a) = self.channels();

		(r, g, b, a)
	}
}

impl<T: Primitive> Pixel<T> for Luma<T> {
	fn from_channels(&self, a: T, _: T, _: T, _: T) -> Luma<T> {
		Luma(a)
	}

	fn to_luma(&self) -> Luma<T> {
		self.clone()
	}

	fn to_luma_alpha(&self) -> LumaA<T> {
		let l = self.channel();

		LumaA(l, Bounded::max_value())
	}

	fn to_rgb(&self) -> Rgb<T> {
		let l1 = self.channel();
		let l2 = self.channel();
		let l3 = self.channel();

		Rgb(l1, l2, l3)
	}

	fn to_rgba(&self) -> Rgba<T> {
		let (r, g, b) = self.to_rgb().channels();

		Rgba(r, g, b, Bounded::max_value())
	}

	fn invert(&mut self) {
		let max: T = Bounded::max_value();
		let l1 = max - self.channel();

		*self = Luma(l1)
	}

	fn map(&self, f: |a: T| -> T) -> Luma<T> {
		let l  = self.channel();
		let l1 = f(l);

		Luma(l1)
	}

	fn map2(&self, other: Luma<T>, f: |a: T, b: T| -> T) -> Luma<T> {
		let l1 = self.channel();
		let l2 = other.channel();

		let l3 = f(l1, l2);

		Luma(l3)
	}

	fn channels4(&self) ->(T, T, T, T) {
		let l = self.channel();
		let max: T = Bounded::max_value();

		(l, max.clone(), max.clone(), max.clone())
	}
}

impl<T: Primitive> Pixel<T> for LumaA<T> {
	fn from_channels(&self, a: T, b: T, _: T, _: T) -> LumaA<T> {
		LumaA(a, b)
	}

	fn to_luma(&self) -> Luma<T> {
		let (l, _) = self.channels();
		Luma(l)
	}

	fn to_luma_alpha(&self) -> LumaA<T> {
		self.clone()
	}

	fn to_rgb(&self) -> Rgb<T> {
		let (l1, _) = self.channels();
		let (l2, _) = self.channels();
		let (l3, _) = self.channels();

		Rgb(l1, l2, l3)
	}

	fn to_rgba(&self) -> Rgba<T> {
		let (r, g, b) = self.to_rgb().channels();
		let a = self.alpha();

		Rgba(r, g, b, a)
	}

	fn invert(&mut self) {
		let l = self.to_luma().channel();
		let a  = self.alpha();

		let max: T = Bounded::max_value();

		*self = LumaA(max - l, a)
	}

	fn map(&self, f: |a: T| -> T) -> LumaA<T> {
		let (l, a) = self.channels();

		let l1 = f(l);

		LumaA(l1, a)
	}

	fn map2(&self, other: LumaA<T>, f: |a: T, b: T| -> T) -> LumaA<T> {
		let (l1, a1) = self.channels();
		let (l2, _)  = other.channels();

		let l3 = f(l1, l2);

		LumaA(l3, a1)
	}

	fn channels4(&self) ->(T, T, T, T) {
		let (l, a) = self.channels();
		let max: T = Bounded::max_value();

		(l, a, max.clone(), max.clone())
	}
}

pub enum PixelBufSlice<'a> {
	Luma8Slice(&'a [Luma<u8>]),
	LumaA8Slice(&'a [LumaA<u8>]),
	Rgb8Slice(&'a [Rgb<u8>]),
	Rgba8Slice(&'a [Rgba<u8>]),
}

pub enum PixelBufMutSlice<'a> {
	Luma8MutSlice(&'a mut [Luma<u8>]),
	LumaA8MutSlice(&'a mut [LumaA<u8>]),
	Rgb8MutSlice(&'a mut [Rgb<u8>]),
	Rgba8MutSlice(&'a mut [Rgba<u8>]),
}

/// An abstraction over a vector of pixel types
#[deriving(Clone, Show, PartialEq)]
pub enum PixelBuf {
	Luma8(Vec<Luma<u8>>),
	//Luma16(Vec<Luma<u16>>),

	LumaA8(Vec<LumaA<u8>>),
	//LumaA16(Vec<LumaA<u16>>),

	Rgb8(Vec<Rgb<u8>>),
	//Rgb16(Vec<Rgb<u16>>),

	Rgba8(Vec<Rgba<u8>>),
	//Rgba16(Vec<Rgba<u16>>),
}

impl PixelBuf {
	pub fn as_luma8<'a>(&'a self) -> Option<&'a [Luma<u8>]> {
		match *self {
			Luma8(ref p) => Some(p.as_slice()),
			_ 	     => None
		}
	}

	pub fn as_luma_alpha8<'a>(&'a self) -> Option<&'a [LumaA<u8>]> {
		match *self {
			LumaA8(ref p) => Some(p.as_slice()),
			_ 	      => None
		}
	}

	pub fn as_rgb8<'a>(&'a self) -> Option<&'a [Rgb<u8>]> {
		match *self {
			Rgb8(ref p) => Some(p.as_slice()),
			_ 	    => None
		}
	}

	pub fn as_rgba8<'a>(&'a self) -> Option<&'a [Rgba<u8>]> {
		match *self {
			Rgba8(ref p) => Some(p.as_slice()),
			_ 	     => None
		}
	}

	pub fn as_slice<'a>(&'a self) -> PixelBufSlice<'a> {
		match *self {
			Luma8(ref p)  => Luma8Slice(p.as_slice()),
			LumaA8(ref p) => LumaA8Slice(p.as_slice()),
			Rgb8(ref p)   => Rgb8Slice(p.as_slice()),
			Rgba8(ref p)  => Rgba8Slice(p.as_slice()),
		}
	}

	pub fn as_mut_slice<'a>(&'a mut self) -> PixelBufMutSlice<'a> {
		match *self {
			Luma8(ref mut p)  => Luma8MutSlice(p.as_mut_slice()),
			LumaA8(ref mut p) => LumaA8MutSlice(p.as_mut_slice()),
			Rgb8(ref mut p)   => Rgb8MutSlice(p.as_mut_slice()),
			Rgba8(ref mut p)  => Rgba8MutSlice(p.as_mut_slice()),
		}
	}

	/// Convert from a vector of bytes to a ```PixelBuf```
	/// Returns ```None``` if the conversion cannot be done.
	pub fn from_bytes(buf: Vec<u8>, color: ColorType) -> Option<PixelBuf> {
		match color {
			colortype::RGB(8) => {
				let p = buf.as_slice()
					   .chunks(3)
					   .map(|a| Rgb::<u8>(a[0], a[1], a[2]))
					   .collect();

				Some(Rgb8(p))
			}

			colortype::RGBA(8) => {
				let p = buf.as_slice()
					   .chunks(4)
					   .map(|a| Rgba::<u8>(a[0], a[1], a[2], a[3]))
					   .collect();

				Some(Rgba8(p))
			}

			colortype::Grey(8) => {
				let p = buf.as_slice()
					   .iter()
					   .map(|a| Luma::<u8>(*a))
					   .collect();

				Some(Luma8(p))
			}

			colortype::GreyA(8) => {
				let p = buf.as_slice()
					   .chunks(2)
					   .map(|a| LumaA::<u8>(a[0], a[1]))
					   .collect();

				Some(LumaA8(p))
			}

			_ => None
		}
	}

	/// Convert from a ```PixelBuf``` to a vector of bytes
	pub fn to_bytes(&self) -> Vec<u8> {
		let mut r = Vec::new();

		match *self {
			Luma8(ref a) => {
				for &i in a.iter() {
					r.push(i.channel());
				}
			}

			LumaA8(ref a) => {
				for &i in a.iter() {
					let (l, a) = i.channels();
					r.push(l);
					r.push(a);
				}
			}

			Rgb8(ref a)  => {
				for &i in a.iter() {
					let (red, g, b) = i.channels();
					r.push(red);
					r.push(g);
					r.push(b);
				}
			}

			Rgba8(ref a) => {
				for &i in a.iter() {
					let (red, g, b, alpha) = i.channels();
					r.push(red);
					r.push(g);
					r.push(b);
					r.push(alpha);
				}
			}
		}

		r
	}
}

/// Convert the ```PixelBuf``` pixels to graysacle
pub fn grayscale(pixels: &PixelBuf) -> PixelBuf {
	match *pixels {
		Luma8(_)      => pixels.clone(),

		LumaA8(ref p) => {
			let n = p.iter().map(|i| i.to_luma()).collect();
			Luma8(n)
		}

		Rgb8(ref p)   => {
			let n = p.iter().map(|i| i.to_luma()).collect();
			Luma8(n)
		}

		Rgba8(ref p)  => {
			let n = p.iter().map(|i| i.to_luma()).collect();
			Luma8(n)
		}
	}
}

fn invert_pixels<A: Primitive, T: Pixel<A>>(pixels: &mut [T]) {
	for i in pixels.mut_iter() {
		i.invert();
	}
}

//TODO: consider implementing a generic map function
//that operates over T: Pixel trait

/// Invert the pixels in ```PixelBuf``` pixels
pub fn invert(pixels: &mut PixelBuf) {
	match *pixels {
		Luma8(ref mut p)  => invert_pixels(p.as_mut_slice()),
		LumaA8(ref mut p) => invert_pixels(p.as_mut_slice()),
		Rgb8(ref mut p)   => invert_pixels(p.as_mut_slice()),
		Rgba8(ref mut p)  => invert_pixels(p.as_mut_slice()),
	}
}

/// Resize this ```PixelBuf``` pixels.
/// ```width``` and ```height``` are the original dimensions.
/// ```nwidth``` and ```nheight``` are the new dimensions.
pub fn resize(pixels:  &PixelBuf,
	      width:   u32,
	      height:  u32,
	      nwidth:  u32,
	      nheight: u32,
	      filter:  sample::FilterType) -> PixelBuf {

	let method = match filter {
		sample::Nearest    => 	sample::Filter {
						kernel:  |x| sample::box_kernel(x),
						support: 0.5
					},
		sample::Triangle   => sample::Filter {
						kernel:  |x| sample::triangle_kernel(x),
						support: 1.0
					},
		sample::CatmullRom => sample::Filter {
						kernel:  |x| sample::catmullrom_kernel(x),
						support: 2.0
					},
		sample::Gaussian   => sample::Filter {
						kernel:  |x| sample::gaussian_kernel(x),
						support: 3.0
					},
		sample::Lanczos3   => sample::Filter {
						kernel:  |x| sample::lanczos3_kernel(x),
						support: 3.0
					},
	};

	let tmp = match *pixels {
		Luma8(ref p)  => Luma8(sample::vertical_sample(p.as_slice(), height, width, nheight, method)),
		LumaA8(ref p) => LumaA8(sample::vertical_sample(p.as_slice(), height, width, nheight, method)),
		Rgb8(ref p)   => Rgb8(sample::vertical_sample(p.as_slice(), height, width, nheight, method)),
		Rgba8(ref p)  => Rgba8(sample::vertical_sample(p.as_slice(), height, width, nheight, method)),
	};

	let method = match filter {
		sample::Nearest    => 	sample::Filter {
						kernel:  |x| sample::box_kernel(x),
						support: 0.5
					},
		sample::Triangle   => sample::Filter {
						kernel:  |x| sample::triangle_kernel(x),
						support: 1.0
					},
		sample::CatmullRom => sample::Filter {
						kernel:  |x| sample::catmullrom_kernel(x),
						support: 2.0
					},
		sample::Gaussian   => sample::Filter {
						kernel:  |x| sample::gaussian_kernel(x),
						support: 3.0
					},
		sample::Lanczos3   => sample::Filter {
						kernel:  |x| sample::lanczos3_kernel(x),
						support: 3.0
					},
	};

	match tmp {
		Luma8(ref p)  => Luma8(sample::horizontal_sample(p.as_slice(), width, nheight, nwidth, method)),
		LumaA8(ref p) => LumaA8(sample::horizontal_sample(p.as_slice(), width, nheight, nwidth, method)),
		Rgb8(ref p)   => Rgb8(sample::horizontal_sample(p.as_slice(), width, nheight, nwidth, method)),
		Rgba8(ref p)  => Rgba8(sample::horizontal_sample(p.as_slice(), width, nheight, nwidth, method)),
	}
}



/// Perfomrs a Gausian blur on this ```Pixelbuf```.
/// ```width``` and ```height``` are the dimensions of the buffer.
/// ```sigma``` is a meausure of how much to blur by.
pub fn blur(pixels:  &PixelBuf,
	    width:   u32,
	    height:  u32,
	    sigma:   f32) -> PixelBuf {

	let sigma = if sigma < 0.0 {
		1.0
	} else {
		sigma
	};

	let method = sample::Filter {
		kernel:  |x| sample::gaussian(x, sigma),
		support: 2.0 * sigma
	};

	let tmp = match *pixels {
		Luma8(ref p)  => Luma8(sample::vertical_sample(p.as_slice(), height, width, height, method)),
		LumaA8(ref p) => LumaA8(sample::vertical_sample(p.as_slice(), height, width, height, method)),
		Rgb8(ref p)   => Rgb8(sample::vertical_sample(p.as_slice(), height, width, height, method)),
		Rgba8(ref p)  => Rgba8(sample::vertical_sample(p.as_slice(), height, width, height, method)),
	};

	let method = sample::Filter {
		kernel:  |x| sample::gaussian(x, sigma),
		support: 2.0 * sigma
	};

	match tmp {
		Luma8(ref p)  => Luma8(sample::horizontal_sample(p.as_slice(), width, height, width, method)),
		LumaA8(ref p) => LumaA8(sample::horizontal_sample(p.as_slice(), width, height, width, method)),
		Rgb8(ref p)   => Rgb8(sample::horizontal_sample(p.as_slice(), width, height, width, method)),
		Rgba8(ref p)  => Rgba8(sample::horizontal_sample(p.as_slice(), width, height, width, method)),
	}
}

fn clamp<N: Num + PartialOrd>(a: N, min: N, max: N) -> N {
	if a > max { max }
	else if a < min { min }
	else { a }
}

fn subtract_pixels<A: Primitive, T: Pixel<A> + Clone>(pixels: &[T], blurred: &mut [T], threshold: i32) {
	let max: A = Bounded::max_value();

	for (p, b) in pixels.iter().zip(blurred.mut_iter()) {
		let a = p.map2(b.clone(), |c, d| {
			let ic = cast::<A, i32>(c).unwrap();
			let id = cast::<A, i32>(d).unwrap();

			let diff = (ic - id).abs();

			if diff > threshold {
				let e = clamp(ic + diff, 0, cast::<A, i32>(max).unwrap());

				cast::<i32, A>(e).unwrap()
			} else {
				c
			}
		});

		*b = a;
	}
}

/// Performs an unsharpen mask on ```pixels```
/// ```sigma``` is the amount to blur the image by.
/// ```threshold``` is a control of how much to sharpen.
/// see https://en.wikipedia.org/wiki/Unsharp_masking#Digital_unsharp_masking
pub fn unsharpen(pixels:    &PixelBuf,
	    	 width:     u32,
	    	 height:    u32,
	    	 sigma:     f32,
	    	 threshold: i32) -> PixelBuf {

	let mut buf = blur(pixels, width, height, sigma);

	{
		let blurred = &mut buf;

		match (pixels, blurred) {
			(&Luma8(ref p), &Luma8(ref mut b)) =>
				subtract_pixels(p.as_slice(), b.as_mut_slice(), threshold),

			(&LumaA8(ref p), &LumaA8(ref mut b)) =>
				subtract_pixels(p.as_slice(), b.as_mut_slice(), threshold),

			(&Rgb8(ref p), &Rgb8(ref mut b)) =>
				subtract_pixels(p.as_slice(), b.as_mut_slice(), threshold),

			(&Rgba8(ref p), &Rgba8(ref mut b)) =>
				subtract_pixels(p.as_slice(), b.as_mut_slice(), threshold),

			(_, _) => fail!("blur operation returned different pixel types")
		}
	}

	buf
}

/// Filters the pixelbuf with the specified 3x3 kernel.
pub fn filter3x3(pixels:  &PixelBuf,
	         width:   u32,
	         height:  u32,
	         kernel:  &[f32]) -> PixelBuf {

	if kernel.len() != 9 {
		return pixels.clone()
	}

	match *pixels {
		Luma8(ref p)  => Luma8(sample::filter_3x3(p.as_slice(), width, height, kernel)),
		LumaA8(ref p) => LumaA8(sample::filter_3x3(p.as_slice(), width, height, kernel)),
		Rgb8(ref p)   => Rgb8(sample::filter_3x3(p.as_slice(), width, height, kernel)),
		Rgba8(ref p)  => Rgba8(sample::filter_3x3(p.as_slice(), width, height, kernel)),
	}
}

fn contrast<A: Primitive, T: Pixel<A>>(p: &[T], contrast: f32) -> Vec<T> {
	let max: A = Bounded::max_value();
	let max = cast::<A, f32>(max).unwrap();

	let percent = ((100.0 + contrast) / 100.0).powi(2);

	p.iter().map(|a| a.map(|b| {
		let c = cast::<A, f32>(b).unwrap();
		let d = ((c / max - 0.5) * percent  + 0.5) * max;
		let e = clamp(d, 0.0, max);

		cast::<f32, A>(e).unwrap()
	})).collect()
}

pub fn adjust_contrast(pixels: &PixelBuf, c: f32) -> PixelBuf {
	match *pixels {
		Luma8(ref p)  => Luma8(contrast(p.as_slice(), c)),
		LumaA8(ref p) => LumaA8(contrast(p.as_slice(), c)),
		Rgb8(ref p)   => Rgb8(contrast(p.as_slice(), c)),
		Rgba8(ref p)  => Rgba8(contrast(p.as_slice(), c)),
	}
}

fn bright<A: Primitive, T: Pixel<A>>(p: &[T], v: i32) -> Vec<T> {
	let max: A = Bounded::max_value();
	let max = cast::<A, i32>(max).unwrap();

	p.iter().map(|a| a.map(|b| {
		let c = cast::<A, i32>(b).unwrap();
		let d = clamp(c + v, 0, max);

		cast::<i32, A>(d).unwrap()
	})).collect()
}

pub fn brighten(pixels: &PixelBuf, c: i32) -> PixelBuf {
	match *pixels {
		Luma8(ref p)  => Luma8(bright(p.as_slice(), c)),
		LumaA8(ref p) => LumaA8(bright(p.as_slice(), c)),
		Rgb8(ref p)   => Rgb8(bright(p.as_slice(), c)),
		Rgba8(ref p)  => Rgba8(bright(p.as_slice(), c)),
	}
}