use crate::drawing::backend::{BackendCoord, DrawingBackend, DrawingErrorKind};
use crate::style::{Color, RGBAColor};
use image::{ImageError, Rgb, RgbImage};

use std::path::Path;

enum Target<'a> {
    File(&'a Path),
    Buffer(&'a mut Vec<u8>),
}

/// The backend that drawing a bitmap
pub struct BitMapBackend<'a> {
    /// The path to the image
    target: Target<'a>,
    /// The image object
    img: RgbImage,
    /// Flag indicates if the bitmap has been saved
    saved: bool,
}

impl<'a> BitMapBackend<'a> {
    /// Create a new bitmap backend
    pub fn new<T: AsRef<Path> + ?Sized>(path: &'a T, dimension: (u32, u32)) -> Self {
        Self {
            target: Target::File(path.as_ref()),
            img: RgbImage::new(dimension.0, dimension.1),
            saved: false,
        }
    }

    /// Create a new bitmap backend which only lives in-memory
    pub fn with_buffer(buf: &'a mut Vec<u8>, dimension: (u32, u32)) -> Self {
        Self {
            target: Target::Buffer(buf),
            img: RgbImage::new(dimension.0, dimension.1),
            saved: false,
        }
    }
}

impl<'a> DrawingBackend for BitMapBackend<'a> {
    type ErrorType = ImageError;

    fn get_size(&self) -> (u32, u32) {
        (self.img.width(), self.img.height())
    }

    fn ensure_prepared(&mut self) -> Result<(), DrawingErrorKind<ImageError>> {
        Ok(())
    }

    fn present(&mut self) -> Result<(), DrawingErrorKind<ImageError>> {
        match &mut self.target {
            Target::File(path) => {
                self.img
                    .save(&path)
                    .map_err(|x| DrawingErrorKind::DrawingError(ImageError::IoError(x)))?;
                self.saved = true;
                Ok(())
            }
            Target::Buffer(target) => {
                let mut actual_img = RgbImage::new(1, 1);
                std::mem::swap(&mut actual_img, &mut self.img);
                target.clear();
                target.append(&mut actual_img.into_raw());
                Ok(())
            }
        }
    }

    fn draw_pixel(
        &mut self,
        point: BackendCoord,
        color: &RGBAColor,
    ) -> Result<(), DrawingErrorKind<ImageError>> {
        if point.0 as u32 >= self.img.width()
            || point.0 < 0
            || point.1 as u32 >= self.img.height()
            || point.1 < 0
        {
            return Ok(());
        }

        let alpha = color.alpha();
        let rgb = color.rgb();

        if alpha >= 1.0 {
            self.img.put_pixel(
                point.0 as u32,
                point.1 as u32,
                Rgb {
                    data: [rgb.0, rgb.1, rgb.2],
                },
            );
        } else {
            let pixel = self.img.get_pixel_mut(point.0 as u32, point.1 as u32);

            let new_color = [rgb.0, rgb.1, rgb.2];

            pixel
                .data
                .iter_mut()
                .zip(&new_color)
                .for_each(|(old, new)| {
                    *old = (f64::from(*old) * (1.0 - alpha) + f64::from(*new) * alpha).min(255.0)
                        as u8;
                });
        }
        Ok(())
    }
}

impl Drop for BitMapBackend<'_> {
    fn drop(&mut self) {
        if !self.saved {
            self.present().expect("Unable to save the bitmap");
        }
    }
}
