#![crate_id = "image"]
#![crate_type = "rlib"]

extern crate flate;
extern crate collections;

pub use ColorType = colortype::ColorType;

pub use JPEGDecoder = jpeg::JPEGDecoder;
pub use JPEGEncoder = jpeg::JPEGEncoder;
pub use PNGDecoder  = png::PNGDecoder;
pub use PNGEncoder  = png::PNGEncoder;
pub use GIFDecoder  = gif::GIFDecoder;
pub use PPMEncoder  = ppm::PPMEncoder;
pub use WebpDecoder = webp::WebpDecoder;

pub mod vp8;
pub mod colortype;

mod jpeg;
mod png;
mod gif;
mod webp;
pub mod hash;
mod ppm;
mod transform;
mod deflate;
mod zlib;
mod lzw;