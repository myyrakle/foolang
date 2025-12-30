pub mod add;
pub mod alloca;
pub mod branch;
pub mod call;
pub mod compare;
pub mod div;
pub mod mul;
pub mod return_;
pub mod sub;

#[derive(Debug)]
pub enum InstructionStatement {
    Call(call::CallInstruction),
    Return(return_::ReturnInstruction),
    Add(add::AddInstruction),
    Sub(sub::SubInstruction),
    Mul(mul::MulInstruction),
    Div(div::DivInstruction),
    Branch(branch::BranchInstruction),
    Jump(branch::JumpInstruction),
    Compare(compare::CompareInstruction),
    Alloca(alloca::AllocaInstruction),
    Load(alloca::LoadInstruction),
    Store(alloca::StoreInstruction),
}
