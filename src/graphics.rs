//! VGA mode 13h 그래픽 (320x200, 256색)
//!
//! 화면 = 0xa0000부터 시작하는 바이트 배열 320*200개
//! 바이트 1개 = 픽셀 1개, 값은 "색 번호"(팔레트 인덱스)
//! 기본 팔레트: 0 검정, 1 파랑, 2 초록, 4 빨강, 12 밝은 빨강, 14 노랑, 15 흰색 ...

pub const WIDTH: usize = 320;
pub const HEIGHT: usize = 200;

const FRAMEBUFFER: *mut u8 = 0xa0000 as *mut u8;

/// (x, y)에 점 하나 찍기. 화면 밖이면 무시
pub fn put_pixel(x: usize, y: usize, color: u8) {
    if x < WIDTH && y < HEIGHT {
        // volatile: 컴파일러가 "아무도 안 읽는 메모리 쓰기"라고 최적화로 지우는 것 방지
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

/// 꽉 찬 사각형
pub fn fill_rect(x: usize, y: usize, w: usize, h: usize, color: u8) {
    for yy in y..(y + h).min(HEIGHT) {
        for xx in x..(x + w).min(WIDTH) {
            put_pixel(xx, yy, color);
        }
    }
}

/// 직선 (브레젠험 알고리즘: 정수 연산만으로 선 긋기)
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

/// 꽉 찬 원 (중심에서 거리 <= r 인 점을 전부 칠함)
pub fn fill_circle(cx: i32, cy: i32, r: i32, color: u8) {
    for dy in -r..=r {
        for dx in -r..=r {
            if dx * dx + dy * dy <= r * r {
                put_pixel_i(cx + dx, cy + dy, color);
            }
        }
    }
}
