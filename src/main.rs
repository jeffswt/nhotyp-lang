use std::collections::HashMap;
use std::env;
use std::error::Error as StdError;
use std::fmt;
use std::fs;
use std::io::Write;
use std::ops;

///////////////////////////////////////////////////////////////////////////////
/// Error handling

#[derive(PartialEq, Eq)]
enum Error {
    IllegalChar { line: usize, value: char },
    TokenTooLong { line: usize, value: usize },
    UnknownToken { line: usize, value: String },
    MalformedAssign { line: usize },
    MalformedCond { line: usize },
    MalformedLoop { line: usize },
    MalformedRet { line: usize },
    MalformedFunc { line: usize },
    MalformedEnd { line: usize },
    UnclosedBlock,
    DuplicateToken { line: usize, value: String },
    WildStatement { line: usize },
    WildFunction { line: usize },
    MisplacedRet { line: usize },
    UndeclaredToken { line: usize, value: String },
    BadExpression { line: usize },
    InputError { line: usize, value: String },
}

impl Error {
    pub fn debug(&self) -> String {
        match self {
            Self::IllegalChar { line, value } => format!("IllegalChar({}, {:?})", line, value),
            Self::TokenTooLong { line, value } => format!("TokenTooLong({}, {})", line, value),
            Self::UnknownToken { line, value } => format!("UnknownToken({}, {:?})", line, value),
            Self::MalformedAssign { line } => format!("MalformedAssign({})", line),
            Self::MalformedCond { line } => format!("MalformedCond({})", line),
            Self::MalformedLoop { line } => format!("MalformedLoop({})", line),
            Self::MalformedRet { line } => format!("MalformedRet({})", line),
            Self::MalformedFunc { line } => format!("MalformedFunc({})", line),
            Self::MalformedEnd { line } => format!("MalformedEnd({})", line),
            Self::UnclosedBlock => format!("UnclosedBlock"),
            Self::DuplicateToken { line, value } => {
                format!("DuplicateToken({}, {:?})", line, value)
            }
            Self::WildStatement { line } => format!("WildStatement({})", line),
            Self::WildFunction { line } => format!("WildFunction({})", line),
            Self::MisplacedRet { line } => format!("MisplacedRet({})", line),
            Self::UndeclaredToken { line, value } => {
                format!("UndeclaredToken({}, {})", line, value)
            }
            Self::BadExpression { line } => format!("BadExpression({})", line),
            Self::InputError { line, value } => format!("InputError({}, {:?})", line, value),
        }
    }

    pub fn format(&self) -> String {
        match self {
            Self::IllegalChar { value, .. } => {
                format!("unexpected character {:?}", value)
            }
            Self::TokenTooLong { value, .. } => {
                format!("token length exceeded ({} of 63)", value)
            }
            Self::UnknownToken { value, .. } => {
                format!("unexpected statement token {:?}", value)
            }
            Self::MalformedAssign { .. } => {
                format!("malformed assignment statement")
            }
            Self::MalformedCond { .. } => {
                format!("malformed conditional statement")
            }
            Self::MalformedLoop { .. } => {
                format!("malformed loop statement")
            }
            Self::MalformedRet { .. } => {
                format!("malformed return statement")
            }
            Self::MalformedFunc { .. } => {
                format!("bad function definition")
            }
            Self::MalformedEnd { .. } => {
                format!("illegal code block end")
            }
            Self::UnclosedBlock => format!("code block unclosed"),
            Self::DuplicateToken { value, .. } => {
                format!("conflict token {:?}", value)
            }
            Self::WildStatement { .. } => {
                format!("statements should appear in functions")
            }
            Self::WildFunction { .. } => {
                format!("function should not appear in functions")
            }
            Self::MisplacedRet { .. } => {
                format!("always return at end of function")
            }
            Self::UndeclaredToken { value, .. } => {
                format!("token {:?} undeclared", value)
            }
            Self::BadExpression { .. } => {
                format!("expression having misplaced tokens")
            }
            Self::InputError { value, .. } => {
                format!("invalid input {:?}", value)
            }
        }
    }

    pub fn line(&self) -> usize {
        match self {
            Self::IllegalChar { line, .. } => *line,
            Self::TokenTooLong { line, .. } => *line,
            Self::UnknownToken { line, .. } => *line,
            Self::MalformedAssign { line, .. } => *line,
            Self::MalformedCond { line, .. } => *line,
            Self::MalformedLoop { line, .. } => *line,
            Self::MalformedRet { line, .. } => *line,
            Self::MalformedFunc { line, .. } => *line,
            Self::MalformedEnd { line, .. } => *line,
            Self::UnclosedBlock => 0,
            Self::DuplicateToken { line, .. } => *line,
            Self::WildStatement { line, .. } => *line,
            Self::WildFunction { line, .. } => *line,
            Self::MisplacedRet { line, .. } => *line,
            Self::UndeclaredToken { line, .. } => *line,
            Self::BadExpression { line, .. } => *line,
            Self::InputError { line, .. } => *line,
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.debug())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.format())
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        "ParserError"
    }

    fn cause(&self) -> Option<&dyn StdError> {
        None
    }
}

///////////////////////////////////////////////////////////////////////////////
/// Tokens and Expressions

#[derive(Eq, PartialEq, Ord, PartialOrd, Hash)]
struct Token {
    value: String,
}

impl Token {
    fn from(s: &str, ptr: usize, allow_const: bool) -> Result<Self, Error> {
        if s.len() > 63 {
            return Err(Error::TokenTooLong {
                line: ptr,
                value: s.len(),
            });
        }
        let x: Vec<_> = s
            .chars()
            .filter(|c| match c {
                '0'..='9' => !allow_const,
                '<' | '=' | '>' => !allow_const,
                '+' | '-' | '*' | '%' | '/' => !allow_const,
                'a'..='z' => false,
                '_' => false,
                _ => true,
            })
            .collect();
        match x.len() {
            0 => Ok(Self {
                value: String::from(s),
            }),
            _ => Err(Error::IllegalChar {
                line: ptr,
                value: x[0],
            }),
        }
    }

    pub fn from_any(ptr: usize, s: &str) -> Result<Self, Error> {
        Self::from(s, ptr, true)
    }

    pub fn from_var(ptr: usize, s: &str) -> Result<Self, Error> {
        Self::from(s, ptr, false)
    }
}

impl Clone for Token {
    fn clone(&self) -> Self {
        Self {
            value: String::from(&self.value),
        }
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.value)
    }
}

struct Expr {
    tokens: Vec<Token>,
}

impl fmt::Debug for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!("{:?}", self.tokens))
    }
}

///////////////////////////////////////////////////////////////////////////////
/// Statements and Nodes

enum Statement {
    Assign {
        var: Token,
        expr: Expr,
        line: usize,
    },
    Cond {
        expr: Expr,
        child: Node,
        line: usize,
    },
    Loop {
        expr: Expr,
        child: Node,
        line: usize,
    },
    Print {
        vars: Vec<Token>,
        line: usize,
    },
    Ret {
        expr: Expr,
        line: usize,
    },
    Func {
        name: Token,
        params: Vec<Token>,
        child: Node,
        line: usize,
    },
}

impl Statement {
    pub fn line(&self) -> usize {
        *match self {
            Self::Assign { line, .. } => line,
            Self::Cond { line, .. } => line,
            Self::Loop { line, .. } => line,
            Self::Print { line, .. } => line,
            Self::Ret { line, .. } => line,
            Self::Func { line, .. } => line,
        }
    }
}

impl fmt::Debug for Statement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Assign { var, expr, line } => {
                f.write_fmt(format_args!("let({:?} <- {:?} @ {})", var, expr, line))
            }
            Self::Cond { expr, child, line } => {
                f.write_fmt(format_args!("if({:?} => {:?} @ {})", expr, child, line))
            }
            Self::Loop { expr, child, line } => {
                f.write_fmt(format_args!("while({:?} => {:?} @ {})", expr, child, line))
            }
            Self::Print { vars, line } => f.write_fmt(format_args!("print({:?} @ {})", vars, line)),
            Self::Ret { expr, line } => f.write_fmt(format_args!("ret({:?} @ {})", expr, line)),
            Self::Func {
                name,
                params,
                child,
                line,
            } => f.write_fmt(format_args!(
                "def({:?} -> {:?} => {:?} @ {})",
                name, params, child, line
            )),
        }
    }
}

struct Node {
    stmts: Vec<Statement>,
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!("{:?}", self.stmts))
    }
}

struct State<'a> {
    lines: Vec<&'a str>,
    ptr: usize,
}

type StmtParseResult = Result<Statement, Error>;

fn parse_stmt_assign(state: &mut State, words: &Vec<&str>) -> StmtParseResult {
    // let <variable> = <expression>
    let len = words.len();
    if len < 4 {
        return Err(Error::MalformedAssign { line: state.ptr });
    }
    let var = Token::from_var(state.ptr, words[1])?;
    let mut tokens = vec![];
    for i in 3..len {
        tokens.push(Token::from_any(state.ptr, words[i])?);
    }
    Ok(Statement::Assign {
        var,
        expr: Expr { tokens },
        line: state.ptr,
    })
}

fn parse_stmt_cond(state: &mut State, words: &Vec<&str>) -> StmtParseResult {
    // if <expression> then
    //     <code block>
    // end if
    let len = words.len();
    if words.len() < 3 || words[len - 1] != "then" {
        return Err(Error::MalformedCond { line: state.ptr });
    }
    // generate expression
    let mut tokens = vec![];
    for i in 1..len - 1 {
        tokens.push(Token::from_any(state.ptr, words[i])?);
    }
    // get child node
    Ok(Statement::Cond {
        expr: Expr { tokens },
        child: parse_node(state, "if")?,
        line: state.ptr,
    })
}

fn parse_stmt_loop(state: &mut State, words: &Vec<&str>) -> StmtParseResult {
    // while <expression> do
    //     <code block>
    // end while
    let len = words.len();
    if words.len() < 3 || words[len - 1] != "do" {
        return Err(Error::MalformedLoop { line: state.ptr });
    }
    // generate expression
    let mut tokens = vec![];
    for i in 1..len - 1 {
        tokens.push(Token::from_any(state.ptr, words[i])?);
    }
    // get child node
    Ok(Statement::Loop {
        expr: Expr { tokens },
        child: parse_node(state, "while")?,
        line: state.ptr,
    })
}

fn parse_stmt_print(state: &mut State, words: &Vec<&str>) -> StmtParseResult {
    // print <var1> <var2> ... <varn>
    // allows 0 variables
    let mut vars = vec![];
    for i in 1..words.len() {
        vars.push(Token::from_var(state.ptr, words[i])?);
    }
    Ok(Statement::Print {
        vars,
        line: state.ptr,
    })
}

fn parse_stmt_ret(state: &mut State, words: &Vec<&str>) -> StmtParseResult {
    // return <expression>
    let len = words.len();
    if len < 2 {
        return Err(Error::MalformedRet { line: state.ptr });
    }
    let mut tokens = vec![];
    for i in 1..len {
        tokens.push(Token::from_any(state.ptr, words[i])?);
    }
    Ok(Statement::Ret {
        expr: Expr { tokens },
        line: state.ptr,
    })
}

fn parse_stmt_func(state: &mut State, words: &Vec<&str>) -> StmtParseResult {
    // function <name> <param1> <param2> ... <paramn> as
    //     <code block>
    // end function
    let len = words.len();
    if words.len() < 3 || words[len - 1] != "as" {
        return Err(Error::MalformedFunc { line: state.ptr });
    }
    // parse parameters
    let name = Token::from_var(state.ptr, words[1])?;
    let mut params = vec![];
    for i in 2..len - 1 {
        let token = Token::from_var(state.ptr, words[i])?;
        if is_reserved_kw(&token.value) {
            return Err(Error::DuplicateToken {
                line: state.ptr,
                value: token.value,
            });
        }
        params.push(token);
    }
    // too many parameters
    if params.len() > 16 {
        return Err(Error::MalformedFunc { line: state.ptr });
    }
    // get child node
    Ok(Statement::Func {
        name,
        params,
        child: parse_node(state, "function")?,
        line: state.ptr,
    })
}

fn parse_stmt(state: &mut State, words: &Vec<&str>) -> StmtParseResult {
    match words[0] {
        "let" => parse_stmt_assign(state, &words),
        "if" => parse_stmt_cond(state, &words),
        "while" => parse_stmt_loop(state, &words),
        "print" => parse_stmt_print(state, &words),
        "return" => parse_stmt_ret(state, &words),
        "function" => parse_stmt_func(state, &words),
        _ => Err(Error::UnknownToken {
            line: state.ptr,
            value: String::from(words[0].to_string()),
        }),
    }
}

fn parse_node(state: &mut State, term: &str) -> Result<Node, Error> {
    let mut stmts = vec![];
    let mut gracefully_ended = term.len() == 0;
    // splitting words here to check for terminations
    while state.ptr < state.lines.len() {
        // eradicate comments
        let mut line = String::from(state.lines[state.ptr]);
        state.ptr += 1;
        if line.contains('#') {
            let splits: Vec<_> = line.split('#').collect();
            line = String::from(splits[0]);
        }
        // filter into singular words and check if is empty line
        let words: Vec<_> = line.split(' ').filter(|w| w.len() > 0).collect();
        if words.len() == 0 {
            continue;
        }
        // 'end' statement triggers code block close
        if words[0] == "end" {
            if words.len() == 2 && words[1] == term {
                gracefully_ended = true;
                break;
            }
            return Err(Error::MalformedEnd { line: state.ptr });
        }
        // send statement to corresponding parser
        stmts.push(parse_stmt(state, &words)?);
    }
    // check if block is unterminated
    if !gracefully_ended {
        return Err(Error::UnclosedBlock);
    }
    // done node parsing
    Ok(Node { stmts })
}

///////////////////////////////////////////////////////////////////////////////
/// Variables

const VARIABLE_LIMIT: i128 = 0x1_0000_0000_0000;

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
struct Variable {
    data: i128,
}

impl Variable {
    fn from(val: i128) -> Self {
        let mut data = val;
        if data > 0 {
            data = data & (VARIABLE_LIMIT - 1);
        } else if data < 0 {
        }
        Self { data }
    }
}

impl ops::Add for Variable {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        Self::from(self.data + other.data)
    }
}

impl ops::Sub for Variable {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        Self::from(self.data - other.data)
    }
}

impl ops::Mul for Variable {
    type Output = Self;
    fn mul(self, other: Self) -> Self::Output {
        Self::from(self.data * other.data)
    }
}

impl ops::Rem for Variable {
    type Output = Self;
    fn rem(self, other: Self) -> Self::Output {
        let a = self.data;
        let b = other.data.abs();
        if b == 0 {
            return Self::from(0);
        }
        Self::from(match a > 0 {
            true => a % b,
            false => (b - (-a) % b) % b,
        })
    }
}

impl ops::Div for Variable {
    type Output = Self;
    fn div(self, other: Self) -> Self::Output {
        let b = other.data.abs();
        if b == 0 {
            return Self::from(0);
        }
        Self::from((self - self % other).data / b)
    }
}

impl ops::BitAnd for Variable {
    type Output = bool;
    fn bitand(self, other: Self) -> bool {
        self.data != 0 && other.data != 0
    }
}

impl ops::BitOr for Variable {
    type Output = bool;
    fn bitor(self, other: Self) -> bool {
        self.data != 0 || other.data != 0
    }
}

impl ops::BitXor for Variable {
    type Output = bool;
    fn bitxor(self, other: Self) -> bool {
        (self.data != 0) ^ (other.data != 0)
    }
}

impl ops::Not for Variable {
    type Output = bool;
    fn not(self) -> bool {
        self.data == 0
    }
}

impl fmt::Debug for Variable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!("{:?}", self.data))
    }
}

///////////////////////////////////////////////////////////////////////////////
/// Program execution

struct Function {
    params: Vec<Token>,
    root: Node,
    line: usize,
}

struct Program {
    funcs: HashMap<Token, Function>,
}

struct RunInstance<'a> {
    prog: &'a Program,
    scope: HashMap<Token, Variable>,
}

fn eval_expr_func(
    instance: &mut RunInstance,
    expr: &Expr,
    ptr: &mut usize,
    line: usize,
) -> Result<Variable, Error> {
    // detect out-of-bounds error
    if *ptr >= expr.tokens.len() {
        return Err(Error::BadExpression { line: line });
    }
    // retrieve function parameter count
    let op_token: &str = &expr.tokens[*ptr].value;
    let op_cnt = match op_token {
        "scan" => 0,
        "+" | "-" | "*" | "%" | "/" => 2,
        "==" | "<" | ">" | "<=" | ">=" | "!=" => 2,
        "and" | "or" | "xor" => 2,
        "not" => 1,
        _ => {
            // parse constant first
            if let Ok(v) = op_token.parse() {
                return Ok(Variable::from(v));
            }
            let token = Token::from_var(line, op_token)?;
            if instance.scope.contains_key(&token) {
                // variable takes precedence
                return Ok(instance.scope[&token].clone());
            } else if instance.prog.funcs.contains_key(&token) {
                // then attempt to call function
                instance.prog.funcs[&token].params.len()
            } else {
                // and nothing else
                return Err(Error::UndeclaredToken {
                    line: line,
                    value: String::from(&token.value),
                });
            }
        }
    };
    // parse parameters
    let mut params = vec![];
    for _ in 0..op_cnt {
        *ptr += 1;
        params.push(eval_expr_func(instance, expr, ptr, line)?);
    }
    // evaluate result
    let v = &params;
    let is = |i: usize| -> bool { v[i].data != 0 };
    Ok(match op_token {
        "scan" => {
            let mut inp = String::new();
            print!(">>> ");
            std::io::stdout().flush().expect("unable to flush stdout");
            if let Err(_) = std::io::stdin().read_line(&mut inp) {
                return Err(Error::InputError {
                    line,
                    value: String::from("null"),
                });
            }
            inp = String::from(inp.trim());
            match inp.parse() {
                Ok(v) => Variable::from(v),
                Err(_) => return Err(Error::InputError { line, value: inp }),
            }
        }
        "+" => v[0] + v[1],
        "-" => v[0] - v[1],
        "*" => v[0] * v[1],
        "%" => v[0] % v[1],
        "/" => v[0] / v[1],
        "==" => Variable::from(if v[0] == v[1] { 1 } else { 0 }),
        "<" => Variable::from(if v[0] < v[1] { 1 } else { 0 }),
        ">" => Variable::from(if v[0] > v[1] { 1 } else { 0 }),
        "<=" => Variable::from(if v[0] <= v[1] { 1 } else { 0 }),
        ">=" => Variable::from(if v[0] >= v[1] { 1 } else { 0 }),
        "!=" => Variable::from(if v[0] != v[1] { 1 } else { 0 }),
        "and" => Variable::from(if is(0) && is(1) { 1 } else { 0 }),
        "or" => Variable::from(if is(0) || is(1) { 1 } else { 0 }),
        "xor" => Variable::from(if is(0) != is(1) { 1 } else { 0 }),
        "not" => Variable::from(if is(0) { 0 } else { 1 }),
        _ => {
            let token = Token::from_var(line, op_token)?;
            call_function(instance.prog, &token, params, line)?
        }
    })
}

fn eval_expr(instance: &mut RunInstance, expr: &Expr, from_line: usize) -> Result<Variable, Error> {
    let mut ptr = 0;
    let res = eval_expr_func(instance, expr, &mut ptr, from_line)?;
    if ptr + 1 < expr.tokens.len() {
        return Err(Error::BadExpression { line: from_line });
    }
    Ok(res)
}

fn is_reserved_kw(token: &str) -> bool {
    match token {
        "and" | "or" | "xor" | "not" | "scan" => true,
        "let" => true,
        "if" | "then" => true,
        "while" | "do" => true,
        "function" | "as" | "return" => true,
        "end" => true,
        "print" => true,
        _ => false,
    }
}

fn exec_statement(instance: &mut RunInstance, stmt: &Statement) -> Result<(), Error> {
    match &stmt {
        &Statement::Assign { var, expr, line } => {
            if is_reserved_kw(&var.value) || instance.prog.funcs.contains_key(&var) {
                return Err(Error::DuplicateToken {
                    line: *line,
                    value: String::from(&var.value),
                });
            }
            let res = eval_expr(instance, &expr, *line)?;
            instance.scope.insert(var.clone(), res);
        }
        &Statement::Cond { expr, child, line } => {
            let cond = eval_expr(instance, &expr, *line)?;
            if cond.data != 0 {
                exec_node(instance, &child)?;
            }
        }
        &Statement::Loop { expr, child, line } => loop {
            let cond = eval_expr(instance, &expr, *line)?;
            if cond.data == 0 {
                break;
            }
            exec_node(instance, &child)?;
        },
        &Statement::Print { vars, line } => {
            print!("...");
            for var in vars {
                if !instance.scope.contains_key(var) {
                    return Err(Error::UndeclaredToken {
                        line: *line,
                        value: String::from(&var.value),
                    });
                }
                let val = &instance.scope[&var];
                print!(" {}", val.data);
            }
            println!("");
            std::io::stdout().flush().expect("unable to flush stdout");
        }
        &Statement::Ret { line, .. } => return Err(Error::MisplacedRet { line: *line }),
        &Statement::Func { line, .. } => return Err(Error::WildFunction { line: *line }),
    }
    Ok(())
}

fn exec_node(instance: &mut RunInstance, node: &Node) -> Result<(), Error> {
    for stmt in &node.stmts {
        match stmt {
            _ => exec_statement(instance, &stmt)?,
        }
    }
    Ok(())
}

fn call_function(
    prog: &Program,
    token: &Token,
    params: Vec<Variable>,
    from_line: usize,
) -> Result<Variable, Error> {
    // lookup function
    if !prog.funcs.contains_key(&token) {
        return Err(Error::UndeclaredToken {
            line: from_line,
            value: String::from(&token.value),
        });
    }
    let func = &prog.funcs[&token];
    // generate instance
    let scope = HashMap::new();
    let mut instance = RunInstance { prog, scope };
    // put parameters into scope
    for i in 0..params.len() {
        let key = func.params[i].clone();
        let value = params[i].clone();
        if prog.funcs.contains_key(&key) {
            return Err(Error::DuplicateToken {
                line: func.line,
                value: key.value,
            });
        }
        instance.scope.insert(key, value);
    }
    // iterate function statements
    let stmts = &func.root.stmts;
    if stmts.len() < 1 {
        return Err(Error::MisplacedRet { line: func.line });
    }
    for i in 0..stmts.len() - 1 {
        exec_statement(&mut instance, &stmts[i])?;
    }
    // last statement must return value
    match &stmts[stmts.len() - 1] {
        Statement::Ret { expr, .. } => eval_expr(&mut instance, &expr, from_line),
        _ => Err(Error::MisplacedRet { line: func.line }),
    }
}

fn run_program(content: &str) -> Result<i64, Error> {
    // parse file for functions
    let mut state = State {
        lines: content.split('\n').collect(),
        ptr: 0,
    };
    let node = parse_node(&mut state, "")?;
    // check for wild statements at global scope and construct program
    let mut prog = Program {
        funcs: HashMap::new(),
    };
    for stmt in node.stmts {
        if let Statement::Func {
            name,
            params,
            child,
            line,
        } = stmt
        {
            if is_reserved_kw(&name.value) || prog.funcs.contains_key(&name) {
                return Err(Error::DuplicateToken {
                    line,
                    value: name.value,
                });
            }
            prog.funcs.insert(
                Token {
                    value: String::from(&name.value),
                },
                Function {
                    params,
                    root: child,
                    line,
                },
            );
        } else {
            return Err(Error::WildStatement { line: stmt.line() });
        }
    }
    // check if main function exists and call
    let main_token = Token {
        value: String::from("main"),
    };
    Ok(call_function(&prog, &main_token, vec![], 0)?.data as i64)
}

fn main() {
    // read program from file
    let args: Vec<_> = env::args().collect();
    if args.len() < 2 {
        eprintln!("nhotyp: fatal error: no input files");
        eprintln!("intepretation terminated.");
        return;
    }
    let filename = &args[1];
    let content;
    match fs::read_to_string(&filename) {
        Ok(v) => content = v,
        Err(_) => {
            eprintln!("nhotyp: fatal error: {}: cannot read file", &filename);
            eprintln!("nhotyp: fatal error: no input files");
            eprintln!("interpretation terminated.");
            return;
        }
    }
    // parse and execute
    match run_program(&content) {
        Ok(v) => std::process::exit((v & 0xffffffffi64) as i32),
        Err(err) => eprintln!("{}:{}: error: {}", &args[1], err.line(), err.format()),
    }
}
