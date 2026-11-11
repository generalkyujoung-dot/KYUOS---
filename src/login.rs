//! 로그인 화면
//!
//! `run()`  : 로그인 창을 띄우고, 맞는 계정이 들어올 때까지 안 끝난다.
//! `welcome()` : 로그인 성공 화면. ESC를 누르면 끝난다(로그아웃).
//!
//! 키 조작: 글자 입력 / Backspace 지우기 / Tab·방향키 칸 이동 / Enter 다음 칸·로그인

use crate::graphics::{self, HEIGHT, WIDTH};
use crate::{interrupts, keyboard};
use pc_keyboard::{DecodedKey, KeyCode};

// ---------- 계정 (파일시스템이 없으니 일단 코드에 박아둠) ----------
const USERNAME: &str = "kyujoung";
const PASSWORD: &str = "osfucked견ㅅ";

// ---------- 색 (VGA 기본 팔레트 번호) ----------
const BLACK: u8 = 0;
const BLUE: u8 = 1;
const GREEN: u8 = 2;
const RED: u8 = 4;
const GRAY: u8 = 7;
const DARK_GRAY: u8 = 8;
const LIGHT_BLUE: u8 = 9;
const YELLOW: u8 = 14;
const WHITE: u8 = 15;

// ---------- 배치 (화면은 320x200, 글자 하나는 8x8) ----------
const WIN_X: usize = 60;
const WIN_Y: usize = 40;
const WIN_W: usize = 200;
const WIN_H: usize = 120;
const TITLE_H: usize = 14;

const LABEL_X: usize = WIN_X + 14;
const FIELD_X: usize = WIN_X + 60;
const FIELD_W: usize = 120;
const FIELD_H: usize = 14;
const USER_Y: usize = WIN_Y + 28;
const PASS_Y: usize = WIN_Y + 50;

const BTN_W: usize = 64;
const BTN_H: usize = 16;
const BTN_X: usize = FIELD_X + (FIELD_W - BTN_W) / 2;
const BTN_Y: usize = WIN_Y + 74;

const MSG_X: usize = WIN_X + 8;
const MSG_Y: usize = WIN_Y + 100;
const MSG_CHARS: usize = 23; // 메시지 줄에 들어가는 최대 글자 수

/// 입력칸 하나에 들어가는 최대 글자 수
pub const MAX_LEN: usize = 14;

/// 고정 크기 입력 버퍼. 힙이 없어서 String 대신 쓴다.
#[derive(Clone, Copy)]
pub struct Input {
    buf: [u8; MAX_LEN],
    len: usize,
}

impl Input {
    pub const fn new() -> Self {
        Input {
            buf: [0; MAX_LEN],
            len: 0,
        }
    }

    /// 글자 추가. 꽉 찼으면 무시한다.
    fn push(&mut self, c: char) {
        if self.len < MAX_LEN && c.is_ascii() {
            self.buf[self.len] = c as u8;
            self.len += 1;
        }
    }

    /// 마지막 글자 지우기
    fn pop(&mut self) {
        if self.len > 0 {
            self.len -= 1;
        }
    }

    fn clear(&mut self) {
        self.len = 0;
    }

    pub fn as_str(&self) -> &str {
        // push에서 ASCII만 받으므로 항상 올바른 문자열
        core::str::from_utf8(&self.buf[..self.len]).unwrap_or("")
    }
}

/// 지금 입력 중인 칸
#[derive(Clone, Copy, PartialEq)]
enum Focus {
    User,
    Pass,
}

impl Focus {
    fn other(self) -> Focus {
        match self {
            Focus::User => Focus::Pass,
            Focus::Pass => Focus::User,
        }
    }
}

/// 로그인 창을 띄우고 로그인에 성공하면 그 사용자 이름을 돌려준다.
pub fn run() -> Input {
    let mut user = Input::new();
    let mut pass = Input::new();
    let mut focus = Focus::User;

    draw_background();
    draw_window();
    draw_fields(&user, &pass, focus);

    let mut cursor_on = true;
    let mut last_blink = interrupts::ticks();

    loop {
        // 1) 쌓인 키를 전부 처리
        let mut changed = false;
        while let Some(key) = keyboard::next_key() {
            changed = true;
            match key {
                // Tab, 위/아래 방향키: 칸 이동
                DecodedKey::Unicode('\t')
                | DecodedKey::RawKey(KeyCode::ArrowUp)
                | DecodedKey::RawKey(KeyCode::ArrowDown) => focus = focus.other(),

                // Enter: 아이디 칸이면 다음 칸으로, 비밀번호 칸이면 로그인 시도
                DecodedKey::Unicode('\n') => match focus {
                    Focus::User => focus = Focus::Pass,
                    Focus::Pass => {
                        draw_button(true); // 눌린 모양
                        if user.as_str() == USERNAME && pass.as_str() == PASSWORD {
                            return user;
                        }
                        pass.clear();
                        draw_message("Wrong user or password", RED);
                        draw_button(false);
                    }
                },

                // Backspace
                DecodedKey::Unicode('\x08') => match focus {
                    Focus::User => user.pop(),
                    Focus::Pass => pass.pop(),
                },

                // 화면에 찍을 수 있는 글자 (스페이스 ~ 물결표)
                DecodedKey::Unicode(c) if (' '..='~').contains(&c) => match focus {
                    Focus::User => user.push(c),
                    Focus::Pass => pass.push(c),
                },

                _ => {}
            }
        }

        // 2) 뭔가 바뀌었으면 입력칸을 다시 그림 (화면 전체가 아니라 칸만 -> 안 깜빡임)
        if changed {
            draw_fields(&user, &pass, focus);
            cursor_on = true;
            last_blink = interrupts::ticks();
        }

        // 3) 커서 깜빡임: 타이머 9틱(약 0.5초)마다 켰다 껐다
        let now = interrupts::ticks();
        if now.wrapping_sub(last_blink) >= 9 {
            cursor_on = !cursor_on;
            last_blink = now;
            let (y, input) = match focus {
                Focus::User => (USER_Y, &user),
                Focus::Pass => (PASS_Y, &pass),
            };
            draw_cursor(y, input.len, cursor_on);
        }

        // 4) 다음 인터럽트(키보드 or 타이머)까지 잠들기
        interrupts::wait_for_interrupt();
    }
}

/// 로그인 성공 화면. ESC를 누르면 돌아온다.
pub fn welcome(user: &Input) {
    graphics::clear(BLUE);

    // 위쪽 바
    graphics::fill_rect(0, 0, WIDTH, 14, GRAY);
    graphics::draw_str(4, 3, "GYUJO OS", BLACK, GRAY);
    let name = user.as_str();
    graphics::draw_str(WIDTH - 4 - name.len() * 8, 3, name, BLUE, GRAY);

    // 가운데 문구: "WELCOME, <이름>!"
    let hello = "WELCOME, ";
    let total = hello.len() + name.len() + 1;
    let x = (WIDTH - total * 8) / 2;
    graphics::draw_str(x, 84, hello, WHITE, BLUE);
    graphics::draw_str(x + hello.len() * 8, 84, name, YELLOW, BLUE);
    graphics::draw_str(x + (hello.len() + name.len()) * 8, 84, "!", WHITE, BLUE);

    draw_centered(104, "LOGIN OK", GREEN, BLUE);
    draw_centered(HEIGHT - 14, "ESC: LOGOUT", GRAY, BLUE);

    loop {
        while let Some(key) = keyboard::next_key() {
            if key == DecodedKey::Unicode('\x1b') {
                return;
            }
        }
        interrupts::wait_for_interrupt();
    }
}

// ==================== 그리기 ====================

fn draw_centered(y: usize, s: &str, fg: u8, bg: u8) {
    graphics::draw_str((WIDTH - s.len() * 8) / 2, y, s, fg, bg);
}

fn draw_background() {
    graphics::clear(BLUE);
    draw_centered(16, "GYUJO OS", WHITE, BLUE);
    draw_centered(HEIGHT - 14, "TAB: SWITCH   ENTER: LOGIN", GRAY, BLUE);
}

/// 입체 테두리. 왼쪽·위를 밝게, 오른쪽·아래를 어둡게 하면 튀어나와 보이고
/// 반대로 하면 들어가 보인다.
fn bevel(x: usize, y: usize, w: usize, h: usize, top_left: u8, bottom_right: u8) {
    graphics::fill_rect(x, y, w, 1, top_left);
    graphics::fill_rect(x, y, 1, h, top_left);
    graphics::fill_rect(x, y + h - 1, w, 1, bottom_right);
    graphics::fill_rect(x + w - 1, y, 1, h, bottom_right);
}

/// 창 틀, 제목, 라벨, 버튼처럼 한 번만 그리면 되는 것들
fn draw_window() {
    // 그림자 -> 창 본체 -> 입체 테두리
    graphics::fill_rect(WIN_X + 4, WIN_Y + 4, WIN_W, WIN_H, BLACK);
    graphics::fill_rect(WIN_X, WIN_Y, WIN_W, WIN_H, GRAY);
    bevel(WIN_X, WIN_Y, WIN_W, WIN_H, WHITE, DARK_GRAY);

    // 제목 줄
    graphics::fill_rect(WIN_X + 2, WIN_Y + 2, WIN_W - 4, TITLE_H, LIGHT_BLUE);
    graphics::draw_str(WIN_X + 6, WIN_Y + 5, "LOGIN", WHITE, LIGHT_BLUE);

    // 라벨 (입력칸 안의 글자와 높이를 맞춤)
    graphics::draw_str(LABEL_X, USER_Y + 3, "USER", BLACK, GRAY);
    graphics::draw_str(LABEL_X, PASS_Y + 3, "PASS", BLACK, GRAY);

    draw_button(false);
}

fn draw_button(pressed: bool) {
    graphics::fill_rect(BTN_X, BTN_Y, BTN_W, BTN_H, GRAY);
    if pressed {
        bevel(BTN_X, BTN_Y, BTN_W, BTN_H, DARK_GRAY, WHITE);
    } else {
        bevel(BTN_X, BTN_Y, BTN_W, BTN_H, WHITE, DARK_GRAY);
    }
    let text = "LOGIN";
    let x = BTN_X + (BTN_W - text.len() * 8) / 2;
    graphics::draw_str(x, BTN_Y + 4, text, BLACK, GRAY);
}

/// 창 아래쪽 메시지 줄. 이전 메시지를 지우고 새로 쓴다.
fn draw_message(msg: &str, color: u8) {
    graphics::fill_rect(MSG_X, MSG_Y, MSG_CHARS * 8, 8, GRAY);
    graphics::draw_str(MSG_X, MSG_Y, msg, color, GRAY);
}

fn draw_fields(user: &Input, pass: &Input, focus: Focus) {
    draw_field(USER_Y, user, false, focus == Focus::User);
    draw_field(PASS_Y, pass, true, focus == Focus::Pass);
}

/// 입력칸 하나. `masked`면 글자 대신 `*`를 찍는다.
fn draw_field(y: usize, input: &Input, masked: bool, focused: bool) {
    // 선택된 칸은 바깥에 파란 테두리, 아니면 창 색으로 덮어서 지움
    let outline = if focused { BLUE } else { GRAY };
    graphics::draw_rect(FIELD_X - 1, y - 1, FIELD_W + 2, FIELD_H + 2, outline);

    // 흰 바탕 + 들어가 보이는 테두리
    graphics::fill_rect(FIELD_X, y, FIELD_W, FIELD_H, WHITE);
    bevel(FIELD_X, y, FIELD_W, FIELD_H, DARK_GRAY, GRAY);

    for (i, &b) in input.buf[..input.len].iter().enumerate() {
        let c = if masked { '*' } else { b as char };
        graphics::draw_char(FIELD_X + 4 + i * 8, y + 3, c, BLACK, WHITE);
    }

    if focused {
        draw_cursor(y, input.len, true);
    }
}

/// 입력 위치에 세로 막대 커서. `on`이 false면 바탕색으로 지운다.
fn draw_cursor(y: usize, len: usize, on: bool) {
    let color = if on { BLACK } else { WHITE };
    graphics::fill_rect(FIELD_X + 4 + len * 8, y + 3, 2, 8, color);
}
