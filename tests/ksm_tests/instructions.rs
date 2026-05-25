#![cfg(feature = "ksm")]
use kerbalobjects::ksm::sections::ArgIndex;
use kerbalobjects::ksm::{Instr, IntSize};
use kerbalobjects::{BufferIterator, Opcode};

const INSTRUCTIONS_BYTES: &[u8] = &[
    0x31, // eof
    0x32, // eop
    0x33, // nop
    0x34, 0x03, // sto
    0x35, // uns
    0x36, 0x56, // gmb
    0x37, 0x12, // smb
    0x38, // gidx
    0x39, // sidx
    0x3a, 0x13, // bfa
    0x3b, 0x10, // jmp
    0x3c, // add
    0x3d, // sub
    0x3e, // mul
    0x3f, // div
    0x40, // pow
    0x41, // cgt
    0x42, // clt
    0x43, // cge
    0x44, // cle
    0x45, // ceq
    0x46, // cne
    0x47, // neg
    0x48, // bool
    0x49, // not
    0x4a, // and
    0x4b, // or
    0x4c, 0x04, 0x06, // call
    0x4d, 0x09, // ret
    0x4e, 0x98, // push
    0x4f, // pop
    0x50, // dup
    0x51, // swap
    0x52, // eval
    0x53, 0x21, 0x27, // addt
    0x54, // rmvt
    0x55, // wait
    0x57, 0x34, // gmet
    0x58, 0x58, // stol
    0x59, 0x08, // stog
    0x5a, 0x41, 0x75, // bscp
    0x5b, 0x08, // escp
    0x5c, 0x26, // stoe
    0x5d, 0x12, 0x14, // phdl
    0x5e, 0x10, // btr
    0x5f, // exst
    0x60, // argb
    0x61, // targ
    0x62, // tcan
    0x63, // jmps
    0xce, 0xa4, // prl
    0xcd, 0xc1, 0xc4, // pdrl
    0xf0, 0x2d, // lbrt
];

const INSTRUCTIONS: &[Instr] = &[
    Instr::ZeroOp(Opcode::Eof),
    Instr::ZeroOp(Opcode::Eop),
    Instr::ZeroOp(Opcode::Nop),
    Instr::OneOp(Opcode::Sto, ArgIndex::from_usize(0x03)),
    Instr::ZeroOp(Opcode::Uns),
    Instr::OneOp(Opcode::Gmb, ArgIndex::from_usize(0x56)),
    Instr::OneOp(Opcode::Smb, ArgIndex::from_usize(0x12)),
    Instr::ZeroOp(Opcode::Gidx),
    Instr::ZeroOp(Opcode::Sidx),
    Instr::OneOp(Opcode::Bfa, ArgIndex::from_usize(0x13)),
    Instr::OneOp(Opcode::Jmp, ArgIndex::from_usize(0x10)),
    Instr::ZeroOp(Opcode::Add),
    Instr::ZeroOp(Opcode::Sub),
    Instr::ZeroOp(Opcode::Mul),
    Instr::ZeroOp(Opcode::Div),
    Instr::ZeroOp(Opcode::Pow),
    Instr::ZeroOp(Opcode::Cgt),
    Instr::ZeroOp(Opcode::Clt),
    Instr::ZeroOp(Opcode::Cge),
    Instr::ZeroOp(Opcode::Cle),
    Instr::ZeroOp(Opcode::Ceq),
    Instr::ZeroOp(Opcode::Cne),
    Instr::ZeroOp(Opcode::Neg),
    Instr::ZeroOp(Opcode::Bool),
    Instr::ZeroOp(Opcode::Not),
    Instr::ZeroOp(Opcode::And),
    Instr::ZeroOp(Opcode::Or),
    Instr::TwoOp(
        Opcode::Call,
        ArgIndex::from_usize(0x04),
        ArgIndex::from_usize(0x06),
    ),
    Instr::OneOp(Opcode::Ret, ArgIndex::from_usize(0x09)),
    Instr::OneOp(Opcode::Push, ArgIndex::from_usize(0x98)),
    Instr::ZeroOp(Opcode::Pop),
    Instr::ZeroOp(Opcode::Dup),
    Instr::ZeroOp(Opcode::Swap),
    Instr::ZeroOp(Opcode::Eval),
    Instr::TwoOp(
        Opcode::Addt,
        ArgIndex::from_usize(0x21),
        ArgIndex::from_usize(0x27),
    ),
    Instr::ZeroOp(Opcode::Rmvt),
    Instr::ZeroOp(Opcode::Wait),
    Instr::OneOp(Opcode::Gmet, ArgIndex::from_usize(0x34)),
    Instr::OneOp(Opcode::Stol, ArgIndex::from_usize(0x58)),
    Instr::OneOp(Opcode::Stog, ArgIndex::from_usize(0x08)),
    Instr::TwoOp(
        Opcode::Bscp,
        ArgIndex::from_usize(0x41),
        ArgIndex::from_usize(0x75),
    ),
    Instr::OneOp(Opcode::Escp, ArgIndex::from_usize(0x08)),
    Instr::OneOp(Opcode::Stoe, ArgIndex::from_usize(0x26)),
    Instr::TwoOp(
        Opcode::Phdl,
        ArgIndex::from_usize(0x12),
        ArgIndex::from_usize(0x14),
    ),
    Instr::OneOp(Opcode::Btr, ArgIndex::from_usize(0x10)),
    Instr::ZeroOp(Opcode::Exst),
    Instr::ZeroOp(Opcode::Argb),
    Instr::ZeroOp(Opcode::Targ),
    Instr::ZeroOp(Opcode::Tcan),
    Instr::ZeroOp(Opcode::Jmps),
    Instr::OneOp(Opcode::Prl, ArgIndex::from_usize(0xa4)),
    Instr::TwoOp(
        Opcode::Pdrl,
        ArgIndex::from_usize(0xc1),
        ArgIndex::from_usize(0xc4),
    ),
    Instr::OneOp(Opcode::Lbrt, ArgIndex::from_usize(0x2d)),
];

#[test]
fn parse_instructions() {
    let mut iter = BufferIterator::new(INSTRUCTIONS_BYTES);

    for true_instr in INSTRUCTIONS {
        let read_instr = Instr::parse(&mut iter, kerbalobjects::ksm::IntSize::One).unwrap();

        assert_eq!(*true_instr, read_instr);
    }
}

#[test]
fn parse_wider_indices() {
    let bytes = &[
        0x4c, 0x00, 0x03, 0x00, 0x08, // call (2-byte index)
        0x58, 0x08, 0x01, 0x71, // stol (3-byte index)
        0x59, 0x21, 0x00, 0x00, 0x00, // stog (4-byte index)
    ];

    let instructions = &[
        Instr::TwoOp(
            Opcode::Call,
            ArgIndex::from_usize(0x0003),
            ArgIndex::from_usize(0x0008),
        ),
        Instr::OneOp(Opcode::Stol, ArgIndex::from_usize(0x080171)),
        Instr::OneOp(Opcode::Stog, ArgIndex::from_usize(0x21000000)),
    ];

    let int_sizes = &[IntSize::Two, IntSize::Three, IntSize::Four];

    let mut iter = BufferIterator::new(bytes);

    for (true_instr, int_size) in instructions.iter().zip(int_sizes) {
        let read_instr = Instr::parse(&mut iter, *int_size).unwrap();

        assert_eq!(*true_instr, read_instr);
    }
}
