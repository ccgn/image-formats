//! Functions for performing affine transformations.

use std::default::Default;

use imaging::pixel::Pixel;
use image::GenericImage;

///Rotate ```pixels``` 90 degrees clockwise.
pub fn rotate90<P: Primitive, T: Pixel<P> + Default + Clone + Copy, I: GenericImage<T>>(
        image:  &I,
        width:  u32,
        height: u32) -> I {

        let (width, height) = image.dimensions();

        let d: T = Default::default();
        let mut out: I = GenericImage::from_pixel(height, width, d);

        for y in range(0, height) {
                for x in range(0, width) {
                        let p = image.get_pixel(x, y);
                        out.put_pixel(height - 1 - y, x, p);
                }
        }

        out
}

///Rotate ```pixels``` 180 degrees clockwise.
pub fn rotate180<P: Primitive, T: Pixel<P> + Default + Copy + Clone, I: GenericImage<T>>(
        image:  &I,
        width:  u32,
        height: u32) -> I {

        let (width, height) = image.dimensions();

        let d: T = Default::default();
        let mut out: I = GenericImage::from_pixel(width, height, d);

        for y in range(0, height) {
                for x in range(0, width) {
                        let p = image.get_pixel(x, y);
                        out.put_pixel(width - 1 - x, height - 1 - y, p);
                }
        }

        out
}

///Rotate ```pixels``` 270 degrees clockwise.
pub fn rotate270<P: Primitive, T: Pixel<P> + Default + Copy + Clone, I: GenericImage<T>>(
        image:  &I,
        width:  u32,
        height: u32) -> I {

        let (width, height) = image.dimensions();

        let d: T = Default::default();
        let mut out: I = GenericImage::from_pixel(height, width, d);

        for y in range(0, height) {
                for x in range(0, width) {
                        let p = image.get_pixel(x, y);
                        out.put_pixel(y, width - 1 - x, p);
                }
        }

        out
}

///Flip ```pixels``` horizontally
pub fn flip_horizontal<P: Primitive, T: Pixel<P> + Default + Copy + Clone, I: GenericImage<T>>(
        image:  &I,
        width:  u32,
        height: u32) -> I {

        let (width, height) = image.dimensions();

        let d: T = Default::default();
        let mut out: I = GenericImage::from_pixel(height, width, d);

        for y in range(0, height) {
                for x in range(0, width) {
                        let p = image.get_pixel(x, y);
                        out.put_pixel(width - 1 - x, y, p);
                }
        }

        out
}

///Flip ```pixels``` vertically
pub fn flip_vertical<P: Primitive, T: Pixel<P> + Default + Copy + Clone, I: GenericImage<T>>(
        image:  &I,
        width:  u32,
        height: u32) -> I {

        let (width, height) = image.dimensions();

        let d: T = Default::default();
        let mut out: I = GenericImage::from_pixel(width, height, d);

        for y in range(0, height) {
                for x in range(0, width) {
                        let p = image.get_pixel(x, y);
                        out.put_pixel(x, height - 1 - y, p);
                }
        }

        out
}