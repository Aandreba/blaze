use blaze_rs::{prelude::*, memobj::MemObjectType};

#[global_context]
static CONTEXT : SimpleContext = SimpleContext::default();

#[cfg(feature = "image")]
#[test]
fn invalid_raw () -> Result<()> {
    use std::ptr::addr_of;
    use ffmpeg_sys_next::{av_pix_fmt_desc_get_id, AVPixelFormat};

    let formats = Global.supported_image_formats(MemAccess::default(), MemObjectType::Image2D).unwrap();
    println!("{:?}", formats);

    for format in formats {
        println!("{format:?}");

        if let Some(desc) = format.ffmpeg_pixel_desc() {
            println!("{format:?}: {desc:?}");
            
            let ffmpeg_format = unsafe {
                av_pix_fmt_desc_get_id(addr_of!(desc))
            };

            if ffmpeg_format != AVPixelFormat::AV_PIX_FMT_NONE {
                println!("{format:?}: {ffmpeg_format:?}");
            }
        }
    }

    Ok(())
}