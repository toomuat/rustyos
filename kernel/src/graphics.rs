use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::Point,
    mono_font::{ascii::FONT_8X13, MonoTextStyle},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle},
    text::{Alignment, LineHeight, Text, TextStyleBuilder},
};
use lazy_static::lazy_static;
use spin::Mutex;

#[derive(Debug, Copy, Clone)]
#[repr(u32)]
pub enum PixelFormat {
    Rgb = 0,
    Bgr,
    Bitmask,
    BltOnly,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(C)]
pub struct PixelBitmask {
    pub red: u32,
    pub green: u32,
    pub blue: u32,
    pub reserved: u32,
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct ModeInfo {
    pub version: u32,
    pub hor_res: u32,
    pub ver_res: u32,
    pub format: PixelFormat,
    pub mask: PixelBitmask,
    pub stride: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FrameBuffer {
    pub base: *mut u8,
    size: usize,
}

lazy_static! {
    pub static ref GOP_DISPLAY: Mutex<Option<GopDisplay>> = Mutex::new(None);
}

pub struct GopDisplay(u64, (usize, usize));

impl DrawTarget for GopDisplay {
    type Error = ();
    type Color = Rgb888;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let hor_res = (self.1).0;
        for Pixel(coord, color) in pixels.into_iter() {
            let x = coord.x as usize;
            let y = coord.y as usize;

            let index = ((x + y * hor_res) * 4) as usize;

            unsafe {
                (self.0 as *mut u8).add(index).write_volatile(color.r());
                (self.0 as *mut u8).add(index + 1).write_volatile(color.g());
                (self.0 as *mut u8).add(index + 2).write_volatile(color.b());
            }
        }

        Ok(())
    }

    fn clear(&mut self, color: Rgb888) -> Result<(), ()> {
        self.fill_solid(&self.bounding_box(), color)
    }

    fn fill_contiguous<I>(
        &mut self,
        area: &embedded_graphics::primitives::Rectangle,
        colors: I,
    ) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        self.draw_iter(
            area.points()
                .zip(colors)
                .map(|(pos, color)| Pixel(pos, color)),
        )
    }

    fn fill_solid(
        &mut self,
        area: &embedded_graphics::primitives::Rectangle,
        color: Self::Color,
    ) -> Result<(), Self::Error> {
        self.fill_contiguous(area, core::iter::repeat(color))
    }
}

impl OriginDimensions for GopDisplay {
    fn size(&self) -> Size {
        Size::new((self.1).0 as u32, (self.1).1 as u32)
    }
}

pub fn initialize(fb: *mut FrameBuffer, mi: *mut ModeInfo) {
    // Fill window black
    let hor_res = unsafe { (*mi).hor_res } as usize;
    let ver_res = unsafe { (*mi).ver_res } as usize;

    GOP_DISPLAY
        .lock()
        .replace(GopDisplay(unsafe { (*fb).base as u64 }, (hor_res, ver_res)));

    for i in 0..hor_res {
        for j in 0..ver_res {
            unsafe {
                (*fb).base.add((i + hor_res * j) * 4).write_volatile(0);
                (*fb).base.add((i + hor_res * j) * 4 + 1).write_volatile(0);
                (*fb).base.add((i + hor_res * j) * 4 + 2).write_volatile(0);
            }
        }
    }

    if let Some(display) = &mut *GOP_DISPLAY.lock() {
        display.clear(RgbColor::WHITE).unwrap();

        // Create a new character style.
        let character_style = MonoTextStyle::new(&FONT_8X13, Rgb888::BLACK);

        // Create a new text style.
        let text_style = TextStyleBuilder::new()
            .alignment(Alignment::Center)
            .line_height(LineHeight::Percent(150))
            .build();

        Text::with_text_style(
            "Hello World!\nThis is rustyos",
            Point::new(display.1 .0 as i32 / 2, display.1 .1 as i32 / 2),
            character_style,
            text_style,
        )
        .draw(display)
        .unwrap();

        let style = PrimitiveStyleBuilder::new()
            .stroke_color(Rgb888::RED)
            .stroke_width(3)
            .fill_color(Rgb888::GREEN)
            .build();

        Rectangle::new(Point::new(30, 20), Size::new(10, 15))
            .into_styled(style)
            .draw(display)
            .unwrap();

        Rectangle::new(Point::new(30, 20), Size::new(10, 15))
            .translate(Point::new(-20, -10))
            .into_styled(style)
            .draw(display)
            .unwrap();
    }
}
