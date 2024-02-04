pub struct Frame {
    data: Vec<u8>
}

impl Frame {
  const WIDTH: usize = 256;
  const HEIGHT: usize = 240;

  pub fn new() -> Frame {
    Frame {
      data: vec![0; Frame::WIDTH * Frame::HEIGHT * 3]
    }
  } 

  pub fn set_pixel(&mut self, x: u8, y: u8, rgb: (u8, u8, u8)) {
    let addr = (y as usize) * Frame::WIDTH + (x as usize);
    self.data[addr] = rgb.0;
    self.data[addr + 1] = rgb.1;
    self.data[addr + 2] = rgb.2;
  }
}