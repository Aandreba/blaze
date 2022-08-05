// https://github.com/leandromoreira/ffmpeg-libav-tutorial#learn-ffmpeg-libav-the-hard-way

pub fn decode_image (path: impl AsRef<Path>) -> Result<Image, Error> {
    let mut file = File::open(path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    let image = Image::decode(&buf)?;
    Ok(image)
}