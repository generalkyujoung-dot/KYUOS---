//! 키보드 입력
//!
//! 인터럽트 핸들러 --(스캔코드)--> 큐 --> 메인 루프가 `next_key()`로 꺼내감
//!
//! 큐는 Mutex 대신 atomic으로 만든 링버퍼다.
//! Mutex를 쓰면 "메인 루프가 잠근 순간 인터럽트가 걸려서 핸들러도 잠그려는"
//! 데드락이 생길 수 있는데, atomic은 잠그는 게 없어서 그럴 일이 없다.
//! (넣는 쪽은 인터럽트 핸들러 하나, 빼는 쪽은 메인 루프 하나라서 이 구조로 충분)

use core::sync::atomic::{AtomicU8, AtomicUsize, Ordering};
use lazy_static::lazy_static;
use pc_keyboard::{DecodedKey, HandleControl, Keyboard, ScancodeSet1, layouts};
use spin::Mutex;

const QUEUE_SIZE: usize = 64;

static QUEUE: [AtomicU8; QUEUE_SIZE] = [const { AtomicU8::new(0) }; QUEUE_SIZE];
static HEAD: AtomicUsize = AtomicUsize::new(0); // 다음에 꺼낼 자리
static TAIL: AtomicUsize = AtomicUsize::new(0); // 다음에 넣을 자리

/// 인터럽트 핸들러가 부른다. 큐가 꽉 차 있으면 그 키는 버린다.
pub fn push_scancode(scancode: u8) {
    let tail = TAIL.load(Ordering::Relaxed);
    let next = (tail + 1) % QUEUE_SIZE;
    if next == HEAD.load(Ordering::Acquire) {
        return;
    }
    QUEUE[tail].store(scancode, Ordering::Relaxed);
    TAIL.store(next, Ordering::Release);
}

fn pop_scancode() -> Option<u8> {
    let head = HEAD.load(Ordering::Relaxed);
    if head == TAIL.load(Ordering::Acquire) {
        return None; // 비어 있음
    }
    let scancode = QUEUE[head].load(Ordering::Relaxed);
    HEAD.store((head + 1) % QUEUE_SIZE, Ordering::Release);
    Some(scancode)
}

lazy_static! {
    // Shift가 눌려 있는지 같은 상태를 기억해야 해서 static으로 둔다.
    // 메인 루프에서만 잠그므로 데드락 걱정 없음.
    static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = Mutex::new(
        Keyboard::new(ScancodeSet1::new(), layouts::Us104Key, HandleControl::Ignore)
    );
}

/// 쌓여 있는 입력에서 "눌린 키" 하나를 꺼낸다. 없으면 `None`.
///
/// 스캔코드 하나가 항상 키 하나는 아니다 (키를 뗄 때도 오고, 방향키는 2바이트).
/// 그래서 키가 완성될 때까지 큐를 계속 읽는다.
pub fn next_key() -> Option<DecodedKey> {
    let mut keyboard = KEYBOARD.lock();
    while let Some(scancode) = pop_scancode() {
        if let Ok(Some(event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(event) {
                return Some(key);
            }
        }
    }
    None
}
