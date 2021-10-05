use crate::token::Literal;

use self::ast::{Binary, Grouping, Unary, Wrapper};

// pg https://www.craftinginterpreters.com/representing-code.html
// i have no clue wtf I'm reading, why use a visitor problem? What does the code do?
// though, as far as I can see, it's implementable in rust https://rust-unofficial.github.io/patterns/patterns/behavioural/visitor.html

// Ok, though I do know that enums make 100% sense in defining expressions.
// so let's do that below

mod ast {
    use crate::token::Literal;
    use crate::token::Token;

    use super::Visitor;

    pub struct Binary {
        pub left: Box<Expr>,
        pub operator: Token,
        pub right: Box<Expr>,
    }

    impl Binary {
        fn new(left: Box<Expr>, operator: Token, right: Box<Expr>) -> Self {
            Binary { left, operator, right }
        }
    }

    pub struct Grouping {
        pub expression: Box<Expr>,
    }

    impl Grouping {
        fn new(expression: Box<Expr>) -> Self {
            Grouping { expression}
        }
    }

    pub struct Unary {
        pub operator: Token,
        pub right: Box<Expr>,
    }

    impl Unary {
        fn new(operator: Token, right: Box<Expr>) -> Self {
            Unary { operator, right }
        }
    }

    pub struct Wrapper<T> {
        pub wrapped: T,
    }

    impl<T> Wrapper<T> {
        pub fn new(wrapped: T) -> Self {
            Wrapper { wrapped }
        }
    }

    pub enum Expr {
        // e.g. expression operator expression
        Literal(Wrapper<Literal>),
        // e.g. "(" expression ")"
        Grouping(Wrapper<Grouping>),
        // e.g. "2323", 123
        Binary(Wrapper<Binary>),
        // e.g. ( "-" | "!" ) expression
        Unary(Wrapper<Unary>),
    }

    impl Expr {
        pub fn accept(&self, visitor: &Visitor) -> String {
            match self {
                Expr::Literal(e) => visitor.visit_literal(&e.wrapped),
                Expr::Grouping(e) => visitor.visit_grouping(&e.wrapped),
                Expr::Binary(e) => visitor.visit_binary(&e.wrapped),
                Expr::Unary(e) => visitor.visit_unary(&e.wrapped),
            }
        }
    }
}

pub struct Visitor;

impl Visitor {
    fn visit_binary(&self, expr: &Binary) -> String {
        format!(
            "{}{}{}",
            expr.left.accept(self),
            expr.operator,
            expr.right.accept(self)
        )
    }
    fn visit_unary(&self, expr: &Unary) -> String {
        format!("{}{}", expr.operator, expr.right.accept(self))
    }
    fn visit_grouping(&self, expr: &Grouping) -> String {
        format!("({})", expr.expression.accept(self))
    }
    fn visit_literal(&self, expr: &Literal) -> String {
        format!("{}", expr)
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn tree() {
        use super::ast::*;
        use crate::token::Literal;
        use crate::token::Token;
        use crate::token::TokenType;
        // create a new tree
        let expression = Expr::Binary(
            Wrapper::new(
                Binary::new(
                    Box::new(
                        Expr::Unary(
                            Token::new(
                                TokenType::Minus, "-".to_string(), Literal::None, 1)
                            Box::new(Expr::)
                            )
                            ))))
            // Box::new(Expr::Unary(
            //     Token::new(TokenType::Minus, "-".to_string(), Literal::None, 1),
            //     Box::new(Expr::Literal(Literal::Number(123.0))),
            // )),
            // Token::new(TokenType::Star, "*".to_string(), Literal::None, 1),
            // Box::new(Expr::Literal(Literal::Number(45.67))),
        );
    }
}

// use crate::token::Token;

// pub struct Expr<T> {
//     pub state: T,
// }

// struct Binary<T, U> {
//     left: Expr<T>,
//     operator: Token,
//     right: Expr<U>,
// }