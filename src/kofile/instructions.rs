use super::symbols::KOSymbol;

pub enum Instr<'a, 'b> {
    ZeroOp,
    OneOp(&'a KOSymbol<'b>),
    TwoOp(&'a KOSymbol<'b>, &'a KOSymbol<'b>),
}
