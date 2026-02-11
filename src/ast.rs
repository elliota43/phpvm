#[derive(Debug, Clone)]
pub enum Expr {
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Null,
    Variable(String),

    BinaryOp {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
    },
    UnaryOp {
        op: UnaryOp,
        expr: Box<Expr>,
    },

    Assign {
        variable: String,
        value: Box<Expr>,
    },

    FunctionCall {
        name: String,
        args: Vec<Expr>,
    },

    ArrayAccess {
        array: Box<Expr>,
        index: Box<Expr>,
    },

    ArrayLiteral(Vec<ArrayEntry>),
}

#[derive(Debug, Clone)]
pub struct ArrayEntry {
    pub key: Option<Expr>,
    pub value: Expr,
}

#[derive(Debug, Clone)]
pub enum BinOp {
    Add, Sub, Mul, Div, Mod,
    Concat,
    Equal, Identical, NotEqual, NotIdentical,
    Less, LessEqual, Greater, GreaterEqual,
    And, Or,
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Negate,
    Not,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Echo(Vec<Expr>),
    ExprStmt(Expr),
    If {
        condition: Expr,
        then_block: Block,
        elseif_blocks: Vec<(Expr, Block)>,
        else_block: Option<Block>,
    },
    While {
        condition: Expr,
        body: Block,
    },
    For {
        init: Option<Expr>,
        condition: Option<Expr>,
        update: Option<Expr>,
        body: Block,
    },
    FunctionDef {
        name: String,
        params: Vec<String>,
        body: Block,
    },
    Return(Option<Expr>),
}

pub type Block = Vec<Stmt>;