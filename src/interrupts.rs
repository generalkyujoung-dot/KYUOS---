//! 인터럽트 처리
//!
//! - CPU 예외: breakpoint, double fault
//! - 하드웨어 인터럽트: 타이머(IRQ0), 키보드(IRQ1)
//!
//! 하드웨어 인터럽트는 8259 PIC 칩 2개를 거쳐서 CPU로 들어온다.
//! 기본 번호(0~15)는 CPU 예외 번호와 겹치므로 32~47번으로 옮겨서 쓴다.

use core::sync::atomic::{AtomicU64, Ordering};
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin::Mutex;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET, // IRQ0
    Keyboard,             // IRQ1
}

/// 부팅 후 타이머 인터럽트가 몇 번 왔는지 (기본 설정으로 1초에 약 18번)
static TICKS: AtomicU64 = AtomicU64::new(0);

pub fn ticks() -> u64 {
    TICKS.load(Ordering::Relaxed)
}

/// 다음 인터럽트가 올 때까지 CPU를 재운다 (`loop {}`처럼 CPU를 100% 쓰지 않음)
pub fn wait_for_interrupt() {
    x86_64::instructions::hlt();
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.double_fault.set_handler_fn(double_fault_handler);
        idt[InterruptIndex::Timer as u8].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard as u8].set_handler_fn(keyboard_interrupt_handler);
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

/// PIC를 초기화하고 CPU의 인터럽트 수신을 켠다. `init_idt` 다음에 불러야 한다.
pub fn init_hardware_interrupts() {
    unsafe { PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    crate::serial_println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    TICKS.fetch_add(1, Ordering::Relaxed);
    // "처리 끝났다"고 PIC에 알려줘야 다음 인터럽트가 온다
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer as u8);
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;

    // 키보드 컨트롤러의 데이터 포트. 여기서 읽어가야 다음 키가 들어온다.
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };

    // 핸들러 안에서는 큐에 넣기만 한다. 해석과 화면 그리기는 메인 루프 몫.
    crate::keyboard::push_scancode(scancode);

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard as u8);
    }
}

#[test_case]
fn test_breakpoint_exception() {
    // invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();
}
