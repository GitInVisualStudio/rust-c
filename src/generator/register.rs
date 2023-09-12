use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};

static REIGSTER_INDEX: AtomicUsize = AtomicUsize::new(8);

#[derive(Clone, Copy, PartialEq)]
pub enum Reg {
    RAX,
    RCX,
    RDX,
    RBX,
    RSI,
    RDI,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
    STACK { offset: usize },
    IMMEDIATE(i64),
    ADDRESS { index: usize, offset: usize },
}

fn get_index(reg: usize) -> Reg {
    match reg {
        0 => Reg::RAX,
        1 => Reg::RCX,
        2 => Reg::RDX,
        3 => Reg::RBX,
        4 => Reg::RSI,
        5 => Reg::RDI,
        6 => Reg::R8,
        7 => Reg::R9,
        8 => Reg::R10,
        9 => Reg::R11,
        10 => Reg::R12,
        11 => Reg::R13,
        12 => Reg::R14,
        13 => Reg::R15,
        _ => Reg::STACK { offset: 0 },
    }
}

static REGSITER_SIZE: AtomicUsize = AtomicUsize::new(8);
static REGISTER_NAMES: [&str; 14] = [
    "rax", "rcx", "rdx", "rbx", "rsi", "rdi", "r8", "r9", "r10", "r11", "r12", "r13", "r14", "r15",
];

impl Reg {
    pub fn push() -> Reg {
        let result = REIGSTER_INDEX.load(Ordering::Relaxed);
        if result == 14 {
            panic!("Out of registers!");
        }
        let result = get_index(result);
        let _ = REIGSTER_INDEX.fetch_add(1, Ordering::Relaxed);
        result
    }

    pub fn pop() -> Reg {
        let result = get_index(REIGSTER_INDEX.load(Ordering::Relaxed));
        let _ = REIGSTER_INDEX.fetch_sub(1, Ordering::Relaxed);
        result
    }

    pub fn current() -> Reg {
        let reg = REIGSTER_INDEX.load(Ordering::Relaxed);
        get_index(reg)
    }

    pub fn set_size(bytes: usize) -> usize {
        let prev = Reg::get_size();
        REGSITER_SIZE.store(bytes, Ordering::Relaxed);
        prev
    }

    pub fn get_size() -> usize {
        REGSITER_SIZE.load(Ordering::Relaxed)
    }

    pub fn get_parameter_index(index: usize) -> Reg {
        match index {
            0 => Reg::RDI,
            1 => Reg::RSI,
            2 => Reg::RDX,
            3 => Reg::RCX,
            4 => Reg::R8,
            5 => Reg::R9,
            _ => panic!("only can pass 6 arguments!"),
        }
    }

    pub fn index(&self) -> usize {
        match self {
            Reg::RAX => 0,
            Reg::RCX => 1,
            Reg::RDX => 2,
            Reg::RBX => 3,
            Reg::RSI => 4,
            Reg::RDI => 5,
            Reg::R8 => 6,
            Reg::R9 => 7,
            Reg::R10 => 8,
            Reg::R11 => 9,
            Reg::R12 => 10,
            Reg::R13 => 11,
            Reg::R14 => 12,
            Reg::R15 => 13,
            Reg::STACK { offset: _ } => 14,
            Reg::IMMEDIATE(_) => 15,
            Reg::ADDRESS {
                index: _,
                offset: _,
            } => 16,
        }
    }

    pub fn as_address(&self) -> Reg {
        Reg::ADDRESS {
            index: self.index(),
            offset: 0,
        }
    }

    pub fn offset(&self, offset: usize) -> Reg {
        match self {
            Reg::ADDRESS { index, offset: _ } => Reg::ADDRESS {
                index: *index,
                offset: offset,
            },
            _ => panic!("cannot create offset on non-address register!"),
        }
    }
}

impl fmt::Display for Reg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut base = "";
        if self.index() < REGISTER_NAMES.len() {
            base = REGISTER_NAMES[self.index()];
        }
        match self {
            Reg::RAX | Reg::RCX | Reg::RDX | Reg::RBX => match Reg::get_size() {
                1 => write!(f, "%{}l", &base[1..2]),
                2 => write!(f, "%{}", &base[1..]),
                4 => write!(f, "%e{}", &base[1..]),
                _ => write!(f, "%{}", &base),
            },
            Reg::RSI | Reg::RDI => match Reg::get_size() {
                1 => write!(f, "%{}l", &base[1..]),
                2 => write!(f, "%{}", &base[1..]),
                4 => write!(f, "%e{}", &base[1..]),
                _ => write!(f, "%{}", &base),
            },
            Reg::STACK { offset } => write!(f, "-{}(%rbp)", offset),
            Reg::IMMEDIATE(value) => write!(f, "${}", value),
            Reg::ADDRESS { index, offset } => {
                let prev = Reg::get_size();
                Reg::set_size(8);
                let register = format!("{}", get_index(*index));
                let result = write!(f, "{}({})", offset, register);
                Reg::set_size(prev);
                result
            }
            _ => match Reg::get_size() {
                1 => write!(f, "%{}b", &base),
                2 => write!(f, "%{}w", &base),
                4 => write!(f, "%{}d", &base),
                _ => write!(f, "%{}", &base),
            },
        }
    }
}
