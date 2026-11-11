#![no_std] // Rust 표준 라이브러리를 링크하지 않도록 합니다
#![no_main] // Rust 언어에서 사용하는 실행 시작 지점 (main 함수)을 사용하지 않습니다
#![feature(custom_test_frameworks)]
#![test_runner(os::test_runner)] // lib.rs의 test_runner 재사용
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use os::graphics::{self, WIDTH};
use os::login;
use os::serial_println;


// mod vga_buffer; mod serial; 은 여기서 다시 선언하면 안 됨 (lib.rs와 중복)

#[unsafe(no_mangle)] // 이 함수의 이름을 mangle하지 않습니다
pub extern "C" fn _start() -> ! {
    // 그래픽 모드에서는 VGA 텍스트 출력(println!)이 안 보이므로 로그는 시리얼로
    serial_println!("Hello World{}", "!");

    os::init();

    #[cfg(test)]
    test_main();

    loop {
        // 로그인에 성공해야 run()이 끝난다
        let user = login::run();
        serial_println!("login ok: {}", user.as_str());

        // ESC를 누르면 welcome()이 끝나고 다시 로그인 화면으로
        login::welcome(&user);
        serial_println!("logout");
    }
}

// 이전 과제의 데모 그림 (지금은 안 쓰지만 남겨둠)
#[allow(dead_code)]
fn draw_demo() {
    graphics::clear(1); // 파란 배경

    
    for x in 0..WIDTH {
        let color = (x * 256 / WIDTH) as u8;
        graphics::fill_rect(x, 0, 1, 20, color);
    }

    graphics::fill_rect(20, 40, 80, 50, 14); //  사각형
    graphics::fill_circle(200, 100, 30, 12); //  원
    graphics::draw_line(0, 199, 319, 30, 15); // 대각선
    graphics::draw_str(10, 30, "HELLO GYUJO", 15, 1);
    graphics::fill_triangle((100,100), (130,130),(70,100), 11);
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("{}", info);
    os::hlt_loop()
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
