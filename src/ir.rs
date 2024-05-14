#[derive(Debug, serde::Deserialize)]
pub struct Module {
    #[serde(rename = "FunctionList")]
    pub funcstions: Vec<Function>,
}

#[derive(Debug, serde::Deserialize)]
pub struct Function {
    #[serde(rename = "GlobalIdentifier")]
    pub name: String,
    #[serde(rename = "Params")]
    pub params: Vec<Param>,
    #[serde(rename = "BasicBlock")]
    pub basic_blocks: Vec<BasicBlock>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
pub enum Name {
    Name(String),
    Number(usize),
}

#[derive(Debug, serde::Deserialize)]
pub struct Type {
    #[serde(rename = "ID")]
    pub id: usize,
    #[serde(rename = "Name")]
    pub name: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct Param {
    #[serde(rename = "Name")]
    pub name: Name,
    #[serde(rename = "Type")]
    pub ty: Type,
}

#[derive(Debug, serde::Deserialize)]
pub struct BasicBlock {
    #[serde(rename = "Name")]
    pub name: Name,
    #[serde(rename = "Instructions")]
    pub insts: Vec<Instruction>,
    #[serde(rename = "Terminator")]
    pub term: Terminator,
}

#[derive(Debug, serde::Deserialize)]
pub struct Operand {
    #[serde(rename = "Constant")]
    pub consant: bool,
    #[serde(rename = "Name")]
    pub name: Option<Name>,
    #[serde(rename = "Type")]
    pub ty: Type,
}

#[derive(Debug, serde::Deserialize)]
pub struct Instruction {
    #[serde(rename = "Dest")]
    pub def: Option<Name>,
    #[serde(rename = "Uses")]
    pub uses: Vec<Operand>,
}

#[derive(Debug, serde::Deserialize)]
pub struct Terminator {
    #[serde(rename = "Dest")]
    pub def: Option<Name>,
    #[serde(rename = "Uses")]
    pub uses: Vec<Operand>,
}
