use std::f64::consts::PI;
use std::fs::write;
use std::io;
use std::io::Write;
use std::mem::transmute;
use std::str::Chars;
use mmap::{MapOption, MemoryMap};

#[derive(Debug)]
struct Tokenizer<'t> {
    iterator: Chars<'t>,
}

impl<'t> Tokenizer<'t> {
    pub fn new(source: &'t str) -> Self {
        Self {
            iterator: source.chars(),
        }
    }

    pub fn next_token(&mut self) -> Result<Option<Token>, String> {
        let Some(next) = self.iterator.next() else {
            return Ok(None);
        };

        match next {
            '(' => Ok(Some(Token::BrOpen)),
            ')' => Ok(Some(Token::BrClose)),
            '^' => Ok(Some(Token::Op(MathOp::Pow))),
            '+' => Ok(Some(Token::Op(MathOp::Add))),
            '-' => Ok(Some(Token::Op(MathOp::Sub))),
            '*' => Ok(Some(Token::Op(MathOp::Mul))),
            '/' => Ok(Some(Token::Op(MathOp::Div))),
            'π' => Ok(Some(Token::Num(PI))),
            ch if ch.is_ascii_digit() || ch == '.' => {
                let mut num = String::from(ch);
                let mut it2 = self.iterator.clone();

                while let Some(next_ch) = it2.next() {
                    if !next_ch.is_ascii_digit() && next_ch != '.' {
                        break;
                    }

                    num.push(next_ch);
                    self.iterator.next();
                }

                Ok(Some(Token::Num(num.parse().map_err(|e| {
                    format!("Encountered error while parsing number: {e}")
                })?)))
            }
            ' ' => self.next_token(),
            _ => Err("Encountered unexpected character".to_string()),
        }
    }
}

/// Parser grammar:
///
/// ```
/// expr = term;
/// term = factor (("+" | "-") factor)*;
/// factor = power (("*" | "/") power)*;
/// power = unary ("^" unary)*;
/// unary = "-"? (unary | primary);
/// primary = Num | "(" expr ")";
/// ```
#[derive(Debug)]
struct Parser<'ecx> {
    tokenizer: Tokenizer<'ecx>,
    current: Option<Token>,
}

impl<'ecx> Parser<'ecx> {
    pub fn new(mut tokenizer: Tokenizer<'ecx>) -> Result<Option<Self>, String> {
        let current = tokenizer.next_token()?;

        Ok(Some(Self { tokenizer, current }))
    }

    pub fn parse(&mut self) -> Result<AST, String> {
        self.term()
    }

    fn term(&mut self) -> Result<AST, String> {
        let mut expr = self.factor()?;

        while self.current == Some(Token::Op(MathOp::Add))
            || self.current == Some(Token::Op(MathOp::Sub))
        {
            let Token::Op(op) = self.current.unwrap() else {
                unreachable!()
            };

            self.next()?;
            expr = AST::Op(Operation {
                lhs: Box::new(expr),
                rhs: Box::new(self.factor()?),
                op,
            })
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<AST, String> {
        let mut expr = self.power()?;

        while self.current == Some(Token::Op(MathOp::Mul))
            || self.current == Some(Token::Op(MathOp::Div))
        {
            let Token::Op(op) = self.current.unwrap() else {
                unreachable!()
            };

            self.next()?;
            expr = AST::Op(Operation {
                lhs: Box::new(expr),
                rhs: Box::new(self.power()?),
                op,
            })
        }

        Ok(expr)
    }

    fn power(&mut self) -> Result<AST, String> {
        let mut expr = self.unary()?;

        while self.current == Some(Token::Op(MathOp::Pow)) {
            self.next()?;
            expr = AST::Op(Operation {
                lhs: Box::new(expr),
                rhs: Box::new(self.unary()?),
                op: MathOp::Pow,
            })
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<AST, String> {
        if matches!(self.current, Some(Token::Op(MathOp::Sub))) {
            let _ = self.next()?;

            let operand = self.unary()?;
            Ok(AST::Op(Operation {
                lhs: Box::new(AST::Num(-1f64)),
                rhs: Box::new(operand),
                op: MathOp::Mul,
            }))
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<AST, String> {
        match self.current {
            Some(Token::Num(num)) => {
                self.next()?;
                Ok(AST::Num(num))
            }
            Some(Token::BrOpen) => {
                // Consume "("
                let _ = self.next()?;

                // Parse inner expression
                let parsed = self.parse();

                // Consume ")"
                let _ = self.expect(Token::BrClose)?;

                parsed
            }
            Some(_) => Err("Unexpected token".to_string()),
            None => Err("Unexpected EOF".to_string()),
        }
    }

    fn expect(&mut self, expected: Token) -> Result<Token, String> {
        if !matches!(self.current, Some(_expected)) {
            Err(format!("Expected {expected:?}, found {:?}", self.current))
        } else {
            Ok(self.current.unwrap())
        }
    }

    fn next(&mut self) -> Result<Option<Token>, String> {
        self.current = self.tokenizer.next_token()?;
        Ok(self.current)
    }
}

#[derive(Debug)]
enum AST {
    Num(f64),
    Op(Operation),
}

#[derive(Debug)]
struct Operation {
    lhs: Box<AST>,
    rhs: Box<AST>,
    op: MathOp,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum Token {
    Num(f64),
    BrOpen,
    BrClose,
    Op(MathOp),
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum MathOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
}

fn gen_backup_xmm0() -> [u8; 9] {
    [
        // sub rsp, 16
        0x48, 0x83, 0xec, 0x10,

        // movdqu XMMWORD PTR [rsp], xmm0
        0xf3, 0x0f, 0x7f, 0x0c, 0x24,
    ]
}

fn gen_restore_xmm0() -> [u8; 9] {
    [
        // movdqu xmm0, XMMWORD PTR [rsp]
        0xf3, 0x0f, 0x6f, 0x0c, 0x24,

        // add rsp, 16
        0x48, 0x83, 0xc4, 0x10,
    ]
}

fn gen_movq_to_xmm0(value: f64) -> [u8; 15] {
    let value = unsafe { transmute::<f64, usize>(value) };
    [
        // movabs r10, value
        0x49, 0xba,
        (value & 0xff) as u8,
        (value >> 8 & 0xff) as u8,
        (value >> 16 & 0xff) as u8,
        (value >> 24 & 0xff) as u8,
        (value >> 32 & 0xff) as u8,
        (value >> 40 & 0xff) as u8,
        (value >> 48 & 0xff) as u8,
        (value >> 56 & 0xff) as u8,

        // movq xmm0, r10
        0x66, 0x49, 0x0f, 0x6e, 0xc2,
    ]
}

fn gen_movq_xmm1_xmm0() -> [u8; 4] {
    [
        // movq xmm1, xmm0
        0xf3, 0x0f, 0x7e, 0xc8
    ]
}

fn gen_addsd() -> [u8; 4] {
    [
        // addsd xmm0, xmm1
        0xf2, 0x0f, 0x58, 0xc1,
    ]
}

fn gen_subsd() -> [u8; 4] {
    [
        // subsd xmm0, xmm1
        0xf2, 0x0f, 0x5c, 0xc1,
    ]
}

fn gen_divsd() -> [u8; 4] {
    [
        // divsd xmm0, xmm1
        0xf2, 0x0f, 0x5e, 0xc1,
    ]
}

fn gen_mulsd() -> [u8; 4] {
    [
        // mulsd xmm0, xmm1
        0xf2, 0x0f, 0x59, 0xc1,
    ]
}

fn codegen_jit(ast: &AST) -> Vec<u8> {
    match ast {
        AST::Num(num) => {
            gen_movq_to_xmm0(*num).to_vec()
        }
        AST::Op(operator) => {
            let mut code = Vec::new();

            code.extend(gen_backup_xmm0());

            code.extend(codegen_jit(&operator.rhs));
            code.extend(gen_movq_xmm1_xmm0());

            code.extend(codegen_jit(&operator.lhs));

            code.extend(match operator.op {
                MathOp::Add => gen_addsd(),
                MathOp::Sub => gen_subsd(),
                MathOp::Mul => gen_mulsd(),
                MathOp::Div => gen_divsd(),
                MathOp::Pow => panic!("Power operator is not supported in JIT mode"),
            });

            code.extend(gen_restore_xmm0());

            code
        }
    }
}

fn jit_compile(ast: &AST) -> Vec<u8> {
    let mut code = Vec::new();

    // Function prologue
    code.extend([
        // push rbp
        0x55,

        // mov rbp, rsp
        0x48, 0x89, 0xe5
    ]);

    // Function body
    code.extend(codegen_jit(ast));

    // Function epilogue
    code.extend([
        // mov rsp, rbp
        0x48, 0x89, 0xec,

        // pop rbp
        0x5d,

        // ret
        0xc3,
    ]);

    code
}

fn execute_code(code: &[u8]) -> f64 {
    use core::arch::asm;

    let map = MemoryMap::new(
        code.len(),
        &[
            MapOption::MapReadable,
            MapOption::MapWritable,
            MapOption::MapExecutable,
        ]
    ).expect("Failed to allocate memory for JIT code");

    unsafe {
        std::ptr::copy(code.as_ptr(), map.data(), code.len());
    }

    unsafe {
        transmute::<_, extern "C" fn()>(map.data())()
    }

    let res: f64;

    unsafe {
        asm!(
            "movq {res}, xmm0",
            res = out(reg) res,
        )
    }

    res
}

fn process_expr(expr: &str) {
    let tokenizer = Tokenizer::new(&expr);
    let mut parser = match Parser::new(tokenizer) {
        Ok(Some(parser)) => parser,
        Ok(None) => return,
        Err(e) => {
            eprintln!("Err: {e}");
            return;
        }
    };

    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("Err: {e}");
            return;
        }
    };

    let compiled_code = jit_compile(&ast);
    write("jit.bin", &compiled_code).unwrap();

    println!("{}", execute_code(&compiled_code));
}

fn take_input() -> String {
    print!("> ");
    io::stdout().flush().unwrap();
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).unwrap();

    buf.trim().to_string()
}

fn main() {
    loop {
        let input = take_input();
        if input == "q" {
            break;
        }

        process_expr(&input);
    }
}
