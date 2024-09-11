use circom_program_structure::ast::ExpressionInfixOpcode;
use serde::{Deserialize, Serialize};
use strum_macros::{Display as StrumDisplay, EnumString};

/// The supported Arithmetic gate types.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, EnumString, StrumDisplay)]
pub enum AGateType {
    AAdd,
    ADiv,
    AEq,
    AGEq,
    AGt,
    ALEq,
    ALt,
    AMul,
    ANeq,
    ASub,
    AXor,
    APow,
    AIntDiv,
    AMod,
    AShiftL,
    AShiftR,
    ABoolOr,
    ABoolAnd,
    ABitOr,
    ABitAnd,
}

impl From<&ExpressionInfixOpcode> for AGateType {
    fn from(opcode: &ExpressionInfixOpcode) -> Self {
        match opcode {
            ExpressionInfixOpcode::Mul => AGateType::AMul,
            ExpressionInfixOpcode::Div => AGateType::ADiv,
            ExpressionInfixOpcode::Add => AGateType::AAdd,
            ExpressionInfixOpcode::Sub => AGateType::ASub,
            ExpressionInfixOpcode::Pow => AGateType::APow,
            ExpressionInfixOpcode::IntDiv => AGateType::AIntDiv,
            ExpressionInfixOpcode::Mod => AGateType::AMod,
            ExpressionInfixOpcode::ShiftL => AGateType::AShiftL,
            ExpressionInfixOpcode::ShiftR => AGateType::AShiftR,
            ExpressionInfixOpcode::LesserEq => AGateType::ALEq,
            ExpressionInfixOpcode::GreaterEq => AGateType::AGEq,
            ExpressionInfixOpcode::Lesser => AGateType::ALt,
            ExpressionInfixOpcode::Greater => AGateType::AGt,
            ExpressionInfixOpcode::Eq => AGateType::AEq,
            ExpressionInfixOpcode::NotEq => AGateType::ANeq,
            ExpressionInfixOpcode::BoolOr => AGateType::ABoolOr,
            ExpressionInfixOpcode::BoolAnd => AGateType::ABoolAnd,
            ExpressionInfixOpcode::BitOr => AGateType::ABitOr,
            ExpressionInfixOpcode::BitAnd => AGateType::ABitAnd,
            ExpressionInfixOpcode::BitXor => AGateType::AXor,
        }
    }
}
