use std::{path::Path};
use ffmpeg_next::{format::{input, context::{input::dump, Input}}, Frame, Packet, packet::Ref, decoder::{Opened, Video, find}};
use ffmpeg_sys_next::{av_find_best_stream};
use crate::{image::{channel::{RawPixel, Rgba}}, buffer::rect::Rect2D};

pub struct ImageRead {
    input: Input,
    dec: Opened,
    frame: Frame,
    packet: Packet,
    idx: i32,
}

impl ImageRead {
    pub fn new (path: impl AsRef<Path>) -> Result<Self, ffmpeg_next::Error> {
        let mut input = input(&path)?;
        dump(&input, 0, path.as_ref().to_str());
    
        let idx = unsafe {
            av_find_best_stream(input.as_mut_ptr(), ffmpeg_sys_next::AVMediaType::AVMEDIA_TYPE_VIDEO, -1, -1, core::ptr::null_mut(), 0)
        };
    
        if idx < 0 {
           return Err(ffmpeg_next::Error::from(idx)) 
        }
    
        let stream = input.stream(0).unwrap();
        let mut ctx = stream.codec();
        ctx.set_parameters(stream.parameters())?;
    
        let codec = find(ctx.id()).ok_or(ffmpeg_next::Error::InvalidData)?;
        let dec = ctx.decoder().open_as(codec)?;

        let frame = unsafe { Frame::empty() };
        let packet = Packet::empty();
    
        Ok(Self {
            input,
            frame,
            dec,
            packet,
            idx,
        })
    }

    #[inline(always)]
    pub fn try_read_frame (&mut self) -> Result<bool, ffmpeg_next::Error> {
        self.packet.read(&mut self.input)?;
        Ok(unsafe { &*self.packet.as_ptr() }.stream_index == self.idx)
    }

    pub fn read_frame (mut self) -> Result<Frame, ffmpeg_next::Error> {
        loop {
            if !self.try_read_frame()? {
                continue;
            }

            self.dec.send_packet(&self.packet)?;
            self.dec.receive_frame(&mut self.frame)?;
            break
        }

        Ok(self.frame)
    }

    pub fn read<P: RawPixel> (self) -> Result<Rect2D<P>, ffmpeg_next::Error> {
        todo!()
    }
}

#[test]
fn test () {
    let read = ImageRead::new("tests/test.png").unwrap();
    let img = read.read::<Rgba<u8>>().unwrap();

    println!("{:?}", &img.as_slice()[..10])
}