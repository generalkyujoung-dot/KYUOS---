//! VGA mode 13h 그래픽 (320x200, 256색)
//!
//! 화면 = 0xa0000부터 시작하는 바이트 배열 320*200개
//! 바이트 1개 = 픽셀 1개, 값은 "색 번호"(팔레트 인덱스)
//! 기본 팔레트: 0 검정, 1 파랑, 2 초록, 4 빨강, 12 밝은 빨강, 14 노랑, 15 흰색 ...

use font8x8::UnicodeFonts;

pub const WIDTH: usize = 320;
pub const HEIGHT: usize = 200;

const FRAMEBUFFER: *mut u8 = 0xa0000 as *mut u8;

#[derive(Clone, Copy)]
pub struct Triangle {
    pub a: (i32,i32),
    pub b: (i32,i32),
    pub c: (i32,i32),
    pub color: u8,
}

impl Triangle {
    pub const fn new(a: (i32,i32), b: (i32,i32), c: (i32,i32), color: u8) -> Self {
        Triangle { a, b, c, color }
    }

    pub fn draw(&self) {
        fill_triangle(self.a, self.b, self.c, self.color);
    }
}

pub fn put_pixel(x: usize, y: usize, color: u8) {
    if x < WIDTH && y < HEIGHT {
        
        unsafe {
            core::ptr::write_volatile(FRAMEBUFFER.add(y * WIDTH + x), color);
        }
    }
}

/// 음수 좌표도 받는 버전 (선/원 그릴 때 화면 밖으로 나가도 안전하게)
fn put_pixel_i(x: i32, y: i32, color: u8) {
    if x >= 0 && y >= 0 {
        put_pixel(x as usize, y as usize, color);
    }
}

/// 화면 전체를 한 색으로
pub fn clear(color: u8) {
    for i in 0..WIDTH * HEIGHT {
        unsafe {
            core::ptr::write_volatile(FRAMEBUFFER.add(i), color);
        }
    }
}


pub fn fill_rect(x: usize, y: usize, w: usize, h: usize, color: u8) {
    for yy in y..(y + h).min(HEIGHT) {
        for xx in x..(x + w).min(WIDTH) {
            put_pixel(xx, yy, color);
        }
    }
}


/// 속이 빈 사각형 (테두리 1픽셀)
pub fn draw_rect(x: usize, y: usize, w: usize, h: usize, color: u8) {
    if w == 0 || h == 0 {
        return;
    }
    fill_rect(x, y, w, 1, color); // 위
    fill_rect(x, y + h - 1, w, 1, color); // 아래
    fill_rect(x, y, 1, h, color); // 왼쪽
    fill_rect(x + w - 1, y, 1, h, color); // 오른쪽
}

pub fn draw_line(mut x0: i32, mut y0: i32, x1: i32, y1: i32, color: u8) {
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        put_pixel_i(x0, y0, color);
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}


pub fn fill_circle(cx: i32, cy: i32, r: i32, color: u8) {
    for dy in -r..=r {
        for dx in -r..=r {
            if dx * dx + dy * dy <= r * r {
                put_pixel_i(cx + dx, cy + dy, color);
            }
        }
    }
}
fn edge(a: (i32, i32), b: (i32, i32), p: (i32, i32)) -> i32 {
    (b.0 - a.0) * (p.1 - a.1) - (b.1 - a.1) * (p.0 - a.0)
}

pub fn fill_triangle(a: (i32, i32), b: (i32, i32), c: (i32, i32), color: u8) {
    // 감싸는 사각형 (.0 = x, .1 = y)
    let min_x = a.0.min(b.0).min(c.0);
    let max_x = a.0.max(b.0).max(c.0);
    let min_y = a.1.min(b.1).min(c.1);
    let max_y = a.1.max(b.1).max(c.1);

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let p = (x, y);
            let w0 = edge(b, c, p);
            let w1 = edge(c, a, p);
            let w2 = edge(a, b, p);
            // 세 변 모두에서 같은 쪽이면 안쪽
            if (w0 >= 0 && w1 >= 0 && w2 >= 0) || (w0 <= 0 && w1 <= 0 && w2 <= 0) {
                put_pixel_i(x, y, color);
            }
        }
    }
}

pub fn draw_char(x: usize, y: usize, c: char, fg: u8, bg: u8) {
    // TODO 1: 비트맵 가져오기
    let glyph = font8x8::BASIC_FONTS
        .get(c)                                       // 글자 c의 [u8; 8] 찾기
        .or_else(|| font8x8::BASIC_FONTS.get('?'))    // 없으면 '?'로
        .unwrap_or([0; 8]);                           // 그것도 없으면 빈칸

    // TODO 2: 8줄 돌기
    for (row, byte) in glyph.iter().enumerate() {
        // TODO 3: 8칸 돌기
        for col in 0..8 {
            // TODO 4: col번째 비트가 1이면 글자색, 0이면 배경색
            let color = if (byte >> col) & 1 == 1 { fg } else { bg };
            put_pixel(x + col, y + row, color);
        }
    }
}
pub fn draw_str(x: usize, y: usize, s: &str, fg: u8, bg: u8) {
    for (i, c) in s.chars().enumerate() {
        draw_char(x + i * 8, y, c, fg, bg);
    }
}
