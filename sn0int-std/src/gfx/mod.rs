use crate::errors::*;
use image::{self, DynamicImage, GenericImageView};
pub use img_hash_median::HashAlg;

pub mod exif;


#[derive(Debug)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Gif,
    WebP,
    Tiff,
    Bmp,
    Ico,
}

impl ImageFormat {
    pub fn mime(&self) -> &str {
        // https://www.iana.org/assignments/media-types/media-types.xhtml#image
        // /etc/nginx/mime.types
        match self {
            ImageFormat::Png => "image/png",
            ImageFormat::Jpeg => "image/jpeg",
            ImageFormat::Gif => "image/gif",
            ImageFormat::WebP => "image/webp",
            ImageFormat::Tiff => "image/tiff",
            ImageFormat::Bmp => "image/bmp",
            ImageFormat::Ico => "image/vnd.microsoft.icon",
        }
    }

    pub fn try_from(format: &image::ImageFormat) -> Result<ImageFormat> {
        use image::ImageFormat::*;
        match format {
            Png => Ok(ImageFormat::Png),
            Jpeg => Ok(ImageFormat::Jpeg),
            Gif => Ok(ImageFormat::Gif),
            WebP => Ok(ImageFormat::WebP),
            Tiff => Ok(ImageFormat::Tiff),
            Bmp => Ok(ImageFormat::Bmp),
            Ico => Ok(ImageFormat::Ico),
            _ => bail!("Unsupported format: {:?}", format),
        }
    }
}

impl From<ImageFormat> for image::ImageFormat {
    fn from(format: ImageFormat) -> image::ImageFormat {
        match format {
            ImageFormat::Png => image::ImageFormat::Png,
            ImageFormat::Jpeg => image::ImageFormat::Jpeg,
            ImageFormat::Gif => image::ImageFormat::Gif,
            ImageFormat::WebP => image::ImageFormat::WebP,
            ImageFormat::Tiff => image::ImageFormat::Tiff,
            ImageFormat::Bmp => image::ImageFormat::Bmp,
            ImageFormat::Ico => image::ImageFormat::Ico,
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

    pub fn perception_hash(&self, hash_alg: HashAlg) -> String {
        let hasher = img_hash_median::HasherConfig::new()
            .hash_alg(hash_alg)
            .to_hasher();
        let hash = hasher.hash_image(&self.image);
        hash.to_base64()
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
    let img_format = image::guess_format(buf)?;
    let format = ImageFormat::try_from(&img_format)?;

    let image = image::load_from_memory_with_format(buf, img_format)?;

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
        let ico = fs::read("../sn0int-registry/assets/favicon.ico").expect("fs::read");
        let img = load(&ico).expect("gfx::load");

        assert_eq!("image/vnd.microsoft.icon", img.mime());
        assert_eq!(16, img.height());
        assert_eq!(16, img.width());
    }
}
