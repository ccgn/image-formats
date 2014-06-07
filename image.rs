use std::io;
use std::slice;
use std::slice::Items;

use std::num::Bounded;

use colortype;
use colortype::ColorType;
use colortype::ConvertColor;
use colortype::{
	RGB,
	RGBA,
	Luma,
	LumaA
};

use png;
use ppm;
use gif;
use jpeg;
use webp;

/// An enumeration of Image Errors
#[deriving(Show, PartialEq, Eq)]
pub enum ImageError {
        ///The Image is not formatted properly
	FormatError,

        ///The Image's dimensions are either too small or too large
	DimensionError,

        ///The Decoder does not support this image format
	UnsupportedError,

        ///The Decoder does not support this color type
	UnsupportedColor,

        ///Not enough data was provided to the Decoder
        ///to decode the image
	NotEnoughData,

        ///An I/O Error occurred while decoding the image
	IoError,

        ///The end of the image has been reached
        ImageEnd
}

pub type ImageResult<T> = Result<T, ImageError>;

/// An enumeration of supported image formats.
/// Not all formats support both encoding and decoding.
pub enum ImageFormat {
	PNG,
	JPEG,
	GIF,
	WEBP,
	PPM
}

/// The trait that all decoders implement
pub trait ImageDecoder {
        ///Return a tuple containing the width and height of the image
        fn dimensions(&mut self) -> ImageResult<(u32, u32)>;

        ///Return the color type of the image e.g RGB(8) (8bit RGB)
        fn colortype(&mut self) -> ImageResult<ColorType>;

        ///Returns the length in bytes of one decoded row of the image
        fn row_len(&mut self) -> ImageResult<uint>;

        ///Read one row from the image into buf
        ///Returns the row index
        fn read_scanline(&mut self, buf: &mut [u8]) -> ImageResult<u32>;

        ///Decode the entire image and return it as a Vector
        fn read_image(&mut self) -> ImageResult<Vec<u8>>;

        ///Decode a specific region of the image, represented by the rectangle
        ///starting from ```x``` and ```y``` and having ```length``` and ```width```
        fn load_rect(&mut self, x: u32, y: u32, length: u32, width: u32) -> ImageResult<Vec<u8>> {
                let (w, h) = try!(self.dimensions());

                if length > h || width > w || x > w || y > h {
                        return Err(DimensionError)
                }

                let c = try!(self.colortype());

                let bpp = colortype::bits_per_pixel(c) / 8;
                let rowlen  = try!(self.row_len());

                let mut buf = Vec::from_elem(length as uint * width as uint * bpp, 0u8);
                let mut tmp = Vec::from_elem(rowlen, 0u8);

                loop {
                        let row = try!(self.read_scanline(tmp.as_mut_slice()));
                        if row - 1 == y {
                                break
                        }
                }

                for i in range(0, length as uint) {
                        {
                                let from = tmp.slice_from(x as uint * bpp)
                                              .slice_to(width as uint * bpp);

                                let to   = buf.mut_slice_from(i * width as uint * bpp)
                                              .mut_slice_to(width as uint * bpp);

                                slice::bytes::copy_memory(to, from);
                        }

                        let _ = try!(self.read_scanline(tmp.as_mut_slice()));
                }

                Ok(buf)
        }
}

#[deriving(Clone, Show, PartialEq)]
enum PixelBuf {
	//Luma8(Vec<Luma<u8>>),
	//Luma16(Vec<Luma<u16>>),

	//LumaA8(Vec<LumaA<u8>>),
	//LumaA16(Vec<LumaA<u16>>),

	RGB8(Vec<RGB<u8>>),
	//RGB16(Vec<RGB<u16>>),

	//RGBA8(Vec<RGBA<u8>>),
	//RGBA16(Vec<RGBA<u16>>),
}

impl<A: Primitive + NumCast + Clone + Bounded, T: ConvertColor<A>> PixelBuf {
	pub fn iter<'a>(&'a self) -> Items<'a, T> {
		match *self {
			//Luma8(ref a)   => a.as_slice().iter(),
			//Luma16(ref a)  => a.as_slice().iter(),

			//LumaA8(ref a)  => a.as_slice().iter(),
			//LumaA16(ref a) => a.as_slice().iter(),

			RGB8(ref a)    => a.as_slice().iter(),
			//RGB16(ref a)   => a.as_slice().iter(),

			//RGBA8(ref a)   => a.as_slice().iter(),
			//RGBA16(ref a)  => a.as_slice().iter(),
			_ => fail!("unimplemented")
		}
	}
}

/// A Generic representation of an image
#[deriving(Clone)]
pub struct Image {
	pixels:  PixelBuf,
	width:   u32,
	height:  u32,
	color:   ColorType,
}

impl Image {
	/// Create a new image from ```r```.
	pub fn load<R: Reader>(r: R, format: ImageFormat) -> ImageResult<Image> {
		match format {
			PNG  => decoder_to_image(png::PNGDecoder::new(r)),
			GIF  => decoder_to_image(gif::GIFDecoder::new(r)),
			JPEG => decoder_to_image(jpeg::JPEGDecoder::new(r)),
			WEBP => decoder_to_image(webp::WebpDecoder::new(r)),
			_    => Err(UnsupportedError),
		}
	}

	/// Create a new image from a byte slice
	pub fn load_from_memory(buf: &[u8], format: ImageFormat) -> ImageResult<Image> {
		let b = io::BufReader::new(buf);

		Image::load(b, format)
	}

	/// Encode this image and write it to ```w```
	pub fn save<W: Writer>(&self, w: W, format: ImageFormat) -> io::IoResult<ImageResult<()>> {
		let r = match format {
			PNG  => {
				let mut p = png::PNGEncoder::new(w);
				try!(p.encode(self.raw_pixels().as_slice(),
					      self.width,
					      self.height,
					      self.color))
				Ok(())
			}
			PPM  => {
				let mut p = ppm::PPMEncoder::new(w);
				try!(p.encode(self.raw_pixels().as_slice(),
					      self.width,
					      self.height,
					      self.color))
				Ok(())
			}
			JPEG => {
				let mut j = jpeg::JPEGEncoder::new(w);
				try!(j.encode(self.raw_pixels().as_slice(),
					      self.width,
					      self.height,
					      self.color))
				Ok(())
			}
			_    => Err(UnsupportedError),
		};

		Ok(r)
	}

	/// Return a reference to the pixel buffer of this image.
	/// Its interpretation is dependent on the image's ```ColorType```.
	pub fn raw_pixels(&self) -> Vec<u8> {
		let mut r = Vec::new();

		match self.pixels {
			//Luma8(ref a) => {
			//	for &i in a.iter() {
			//		r.push(i.channel());
			//	}
			//}
			RGB8(ref a)  => {
				for &i in a.iter() {
					let (red, g, b) = i.channels();
					r.push(red);
					r.push(g);
					r.push(b);
				}
			}
			//RGBA8(ref a) => {
			//	for &i in a.iter() {
			//		let (red, g, b, alpha) = i.channels();
			//		r.push(red);
			//		r.push(g);
			//		r.push(b);
			//		r.push(alpha);
			//	}
			//}

			_  => fail!("unimplemented")
		}

		r
	}

	/// Returns a tuple of the image's width and height.
	pub fn dimensions(&self) -> (u32, u32) {
		(self.width, self.height)
	}

	/// The colortype of this image.
	pub fn colortype(&self) -> ColorType {
		self.color
	}

	pub fn grayscale(&self) -> Image {
		let pixels = self.pixels
				 .iter()
				 .map(|p| Luma(p.to_luma().channel() as u8))
				 .collect();

		Image {
			pixels: Luma8(pixels),
			width:  self.width,
			height: self.height,
			color:  colortype::Grey(8),
		}
	}
}

fn decoder_to_image<I: ImageDecoder>(codec: I) -> ImageResult<Image> {
	let mut codec = codec;

	let color  = try!(codec.colortype());
	let buf    = try!(codec.read_image());
	let (w, h) = try!(codec.dimensions());

	let pixels = match color {
		colortype::Rgb(8) => {
			let p = buf.as_slice()
				   .chunks(3)
				   .map(|a| RGB::<u8>(a[0], a[1], a[2]))
				   .collect();
			RGB8(p)
		}

		/*colortype::Rgba(8) => {
			let p = buf.as_slice()
				   .chunks(4)
				   .map(|a| RGBA::<u8>(a[0], a[1], a[2], a[3]))
				   .collect();

			RGBA8(p)
		}

		colortype::Grey(8) => {
			let p = buf.as_slice()
				   .map(|a| Luma::<u8>(a))
				   .collect();
			Luma8(p)
		}*/

		_ => return Err(UnsupportedColor)
	};

	let im = Image {
		pixels:  pixels,
		width:   w,
		height:  h,
		color:   color,
	};

	Ok(im)
}

fn clamp<N: Num + Ord>(n: N, min: N, max: N) -> N {
	if n < min {
		min
	} else if n > max {
		max
	} else {
		n
	}
}