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

impl From<call::CallInstruction> for InstructionStatement {
    fn from(instr: call::CallInstruction) -> Self {
        InstructionStatement::Call(instr)
    }
}

impl From<return_::ReturnInstruction> for InstructionStatement {
    fn from(instr: return_::ReturnInstruction) -> Self {
        InstructionStatement::Return(instr)
    }
}

impl From<add::AddInstruction> for InstructionStatement {
    fn from(instr: add::AddInstruction) -> Self {
        InstructionStatement::Add(instr)
    }
}

impl From<sub::SubInstruction> for InstructionStatement {
    fn from(instr: sub::SubInstruction) -> Self {
        InstructionStatement::Sub(instr)
    }
}

impl From<mul::MulInstruction> for InstructionStatement {
    fn from(instr: mul::MulInstruction) -> Self {
        InstructionStatement::Mul(instr)
    }
}

impl From<div::DivInstruction> for InstructionStatement {
    fn from(instr: div::DivInstruction) -> Self {
        InstructionStatement::Div(instr)
    }
}

impl From<branch::BranchInstruction> for InstructionStatement {
    fn from(instr: branch::BranchInstruction) -> Self {
        InstructionStatement::Branch(instr)
    }
}

impl From<branch::JumpInstruction> for InstructionStatement {
    fn from(instr: branch::JumpInstruction) -> Self {
        InstructionStatement::Jump(instr)
    }
}

impl From<compare::CompareInstruction> for InstructionStatement {
    fn from(instr: compare::CompareInstruction) -> Self {
        InstructionStatement::Compare(instr)
    }
}

impl From<alloca::AllocaInstruction> for InstructionStatement {
    fn from(instr: alloca::AllocaInstruction) -> Self {
        InstructionStatement::Alloca(instr)
    }
}

impl From<alloca::LoadInstruction> for InstructionStatement {
    fn from(instr: alloca::LoadInstruction) -> Self {
        InstructionStatement::Load(instr)
    }
}

impl From<alloca::StoreInstruction> for InstructionStatement {
    fn from(instr: alloca::StoreInstruction) -> Self {
        InstructionStatement::Store(instr)
    }
}
