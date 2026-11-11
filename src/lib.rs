#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;

// 모듈은 lib.rs에서만 선언하고, main.rs는 os::... 로 가져다 씀
// pub이어야 main.rs에서 println!/serial_println! 매크로가 동작함
pub mod graphics;
pub mod interrupts;
pub mod keyboard;
pub mod login;
pub mod serial;
pub mod vga_buffer;

pub fn init() {
    interrupts::init_idt();
    interrupts::init_hardware_interrupts(); // 타이머 + 키보드
}

/// 할 일이 없을 때 CPU를 재우면서 무한 대기
pub fn hlt_loop() -> ! {
    loop {
        interrupts::wait_for_interrupt();
    }
}

// ---------- 테스트 프레임워크 (main.rs도 같이 씀) ----------

pub trait Testable {
    fn run(&self);
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

// #[cfg(test)]를 빼야 main.rs의 테스트에서 os::test_runner로 쓸 수 있음
pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

// ---------- `cargo test --lib` 전용 ----------

/// Entry point for `cargo test`
#[cfg(test)]
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    init();
    test_main();
    loop {}
}

// 일반 모드 panic handler는 main.rs가 담당 (여기 두면 main.rs 테스트 실패 시 멈춰버림)
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}
