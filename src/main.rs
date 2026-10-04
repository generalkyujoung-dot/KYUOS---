#![no_std] // Rust 표준 라이브러리를 링크하지 않도록 합니다
#![no_main] // Rust 언어에서 사용하는 실행 시작 지점 (main 함수)을 사용하지 않습니다
#![feature(custom_test_frameworks)]
#![test_runner(os::test_runner)] // lib.rs의 test_runner 재사용
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use os::graphics::{self, WIDTH};
use os::serial_println;

// mod vga_buffer; mod serial; 은 여기서 다시 선언하면 안 됨 (lib.rs와 중복)

#[unsafe(no_mangle)] // 이 함수의 이름을 mangle하지 않습니다
pub extern "C" fn _start() -> ! {
    // 그래픽 모드에서는 VGA 텍스트 출력(println!)이 안 보이므로 로그는 시리얼로
    serial_println!("Hello World{}", "!");

    os::init();

    #[cfg(test)]
    test_main();

    draw_demo();

    serial_println!("It did not crash!");
    loop {}
}

fn draw_demo() {
    graphics::clear(1); // 파란 배경

    // 상단: 기본 팔레트 256색을 띠로 표시
    for x in 0..WIDTH {
        let color = (x * 256 / WIDTH) as u8;
        graphics::fill_rect(x, 0, 1, 20, color);
    }

    graphics::fill_rect(20, 40, 80, 50, 14); // 노란 사각형
    graphics::fill_circle(200, 100, 30, 12); // 빨간 원
    graphics::draw_line(0, 199, 319, 30, 15); // 흰 대각선
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os::test_panic_handler(info)
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
