//! BNF grammar:
//!
//! ```text
//! condition      = value ("==" | ">" ...) value
//! to             = value value TO value
//! ```

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::cut;
use nom::sequence::tuple;
use nom::IResult;
use Condition::*;

use crate::{parse_value, FPError, FilterCondition, Span, Token};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Condition<'a> {
    GreaterThan(Token<'a>),
    GreaterThanOrEqual(Token<'a>),
    Equal(Token<'a>),
    NotEqual(Token<'a>),
    LowerThan(Token<'a>),
    LowerThanOrEqual(Token<'a>),
    Between { from: Token<'a>, to: Token<'a> },
}

impl<'a> Condition<'a> {
    /// This method can return two operations in case it must express
    /// an OR operation for the between case (i.e. `TO`).
    pub fn negate(self) -> (Self, Option<Self>) {
        match self {
            GreaterThan(n) => (LowerThanOrEqual(n), None),
            GreaterThanOrEqual(n) => (LowerThan(n), None),
            Equal(s) => (NotEqual(s), None),
            NotEqual(s) => (Equal(s), None),
            LowerThan(n) => (GreaterThanOrEqual(n), None),
            LowerThanOrEqual(n) => (GreaterThan(n), None),
            Between { from, to } => (LowerThan(from), Some(GreaterThan(to))),
        }
    }
}

/// condition      = value ("==" | ">" ...) value
pub fn parse_condition<'a, E: FPError<'a>>(
    input: Span<'a>,
) -> IResult<Span<'a>, FilterCondition, E> {
    let operator = alt((tag("<="), tag(">="), tag("!="), tag("<"), tag(">"), tag("=")));
    let (input, (key, op, value)) = tuple((|c| parse_value(c), operator, cut(parse_value)))(input)?;

    let fid = key;

    match *op.fragment() {
        "=" => {
            let k = FilterCondition::Condition { fid, op: Equal(value) };
            Ok((input, k))
        }
        "!=" => {
            let k = FilterCondition::Condition { fid, op: NotEqual(value) };
            Ok((input, k))
        }
        ">" | "<" | "<=" | ">=" => {
            let k = match *op.fragment() {
                ">" => FilterCondition::Condition { fid, op: GreaterThan(value) },
                "<" => FilterCondition::Condition { fid, op: LowerThan(value) },
                "<=" => FilterCondition::Condition { fid, op: LowerThanOrEqual(value) },
                ">=" => FilterCondition::Condition { fid, op: GreaterThanOrEqual(value) },
                _ => unreachable!(),
            };
            Ok((input, k))
        }
        _ => unreachable!(),
    }
}

/// to             = value value TO value
pub fn parse_to<'a, E: FPError<'a>>(input: Span<'a>) -> IResult<Span, FilterCondition, E> {
    let (input, (key, from, _, to)) =
        tuple((|c| parse_value(c), |c| parse_value(c), tag("TO"), cut(parse_value)))(input)?;

    Ok((
        input,
        FilterCondition::Condition {
            fid: key.into(),
            op: Between { from: from.into(), to: to.into() },
        },
    ))
}
