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

pub fn initialize(fb: *mut FrameBuffer, mi: *mut ModeInfo) {
    // Fill window black
    let hor_res = unsafe { (*mi).hor_res } as usize;
    let ver_res = unsafe { (*mi).ver_res } as usize;

    for i in 0..hor_res {
        for j in 0..ver_res {
            unsafe {
                (*fb).base.add((i + hor_res * j) * 4).write_volatile(0);
                (*fb).base.add((i + hor_res * j) * 4 + 1).write_volatile(0);
                (*fb).base.add((i + hor_res * j) * 4 + 2).write_volatile(0);
            }
        }
    }
}
