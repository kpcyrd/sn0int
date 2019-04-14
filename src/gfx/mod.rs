use crate::errors::*;
use image::{self, DynamicImage, GenericImageView};

pub mod exif;


#[derive(Debug)]
pub enum ImageFormat {
    PNG,
    JPEG,
    GIF,
    WEBP,
    TIFF,
    BMP,
    ICO,
}

impl ImageFormat {
    pub fn mime(&self) -> &str {
        // https://www.iana.org/assignments/media-types/media-types.xhtml#image
        // /etc/nginx/mime.types
        match self {
            ImageFormat::PNG => "image/png",
            ImageFormat::JPEG => "image/jpeg",
            ImageFormat::GIF => "image/gif",
            ImageFormat::WEBP => "image/webp",
            ImageFormat::TIFF => "image/tiff",
            ImageFormat::BMP => "image/bmp",
            ImageFormat::ICO => "image/vnd.microsoft.icon",
        }
    }

    pub fn try_from(format: &image::ImageFormat) -> Result<ImageFormat> {
        use image::ImageFormat::*;
        match format {
            PNG => Ok(ImageFormat::PNG),
            JPEG => Ok(ImageFormat::JPEG),
            GIF => Ok(ImageFormat::GIF),
            WEBP => Ok(ImageFormat::WEBP),
            TIFF => Ok(ImageFormat::TIFF),
            BMP => Ok(ImageFormat::BMP),
            ICO => Ok(ImageFormat::ICO),

            HDR | PNM | TGA => bail!("Unsupported format: {:?}", format),
        }
    }
}

impl Into<image::ImageFormat> for ImageFormat {
    fn into(self) -> image::ImageFormat {
        match self {
            ImageFormat::PNG => image::ImageFormat::PNG,
            ImageFormat::JPEG => image::ImageFormat::JPEG,
            ImageFormat::GIF => image::ImageFormat::GIF,
            ImageFormat::WEBP => image::ImageFormat::WEBP,
            ImageFormat::TIFF => image::ImageFormat::TIFF,
            ImageFormat::BMP => image::ImageFormat::BMP,
            ImageFormat::ICO => image::ImageFormat::ICO,
        }
    }
}

pub struct Image {
    image: DynamicImage,
    format: ImageFormat,
}

impl Image {
    #[inline]
    pub fn mime(&self) -> &str {
        self.format.mime()
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.image.width()
    }

    #[inline]
    pub fn height(&self) -> u32 {
        self.image.height()
    }
}

impl AsRef<DynamicImage> for Image {
    fn as_ref(&self) -> &DynamicImage {
        &self.image
    }
}

#[inline]
pub fn guess_format(buf: &[u8]) -> Result<ImageFormat> {
    let format = image::guess_format(buf)?;
    ImageFormat::try_from(&format)
}

pub fn load(buf: &[u8]) -> Result<Image> {
    let img_format = image::guess_format(&buf)?;
    let format = ImageFormat::try_from(&img_format)?;

    let image = image::load_from_memory_with_format(&buf, img_format)?;

    Ok(Image {
        image,
        format,
    })
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn verify_gfx_load_ico() {
        let ico = fs::read("./sn0int-registry/assets/favicon.ico").expect("fs::read");
        let img = load(&ico).expect("gfx::load");

        assert_eq!("image/vnd.microsoft.icon", img.mime());
        assert_eq!(16, img.height());
        assert_eq!(16, img.width());
    }
}
