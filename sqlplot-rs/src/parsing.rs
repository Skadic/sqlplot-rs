use nom::{
    bytes::complete::{tag, take_till1},
    character::complete::{self as character, multispace0, space1},
    combinator::{opt, recognize},
    multi::separated_list1,
    number::complete as number,
    sequence::{delimited, separated_pair},
    IResult, Parser,
};

#[derive(Debug, Clone, PartialEq)]
pub struct ResultLineItem<'a> {
    pub ident: &'a str,
    pub value: ResultLineItemValue<'a>,
}

impl<'a> ResultLineItem<'a> {
    fn new(ident: &'a str, value: ResultLineItemValue<'a>) -> Self {
        Self { ident, value }
    }

    fn int(ident: &'a str, value: i64) -> Self {
        Self::new(ident, ResultLineItemValue::Int(value))
    }

    fn float(ident: &'a str, value: f64) -> Self {
        Self::new(ident, ResultLineItemValue::Float(value))
    }

    fn string(ident: &'a str, value: &'a str) -> Self {
        Self::new(ident, ResultLineItemValue::Str(value))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResultLineItemValue<'a> {
    Str(&'a str),
    Int(i64),
    Float(f64),
}

pub fn parse_result_line(input: &str) -> IResult<&str, Vec<ResultLineItem>> {
    delimited(
        tag("RESULT "),
        separated_list1(space1, parse_item),
        opt(multispace0),
    )(input)
}

fn parse_item(input: &str) -> IResult<&str, ResultLineItem> {
    // and item is an identifier and some content separated by an equals sign
    let (rest, (ident, value)) = separated_pair(
        take_till1(|c| c == '='),
        character::char('='),
        parse_item_value,
    )(input)?;
    Ok((rest, ResultLineItem { ident, value }))
}

fn parse_item_value(input: &str) -> IResult<&str, ResultLineItemValue> {
    let str_parser = delimited(
        character::char('"'),
        take_till1(|c| c == '"'),
        character::char('"'),
    )
    .map(ResultLineItemValue::Str);

    let num_parser = recognize(number::double).map(|num: &str| {
        if let Ok(n) = num.parse::<i64>() {
            return ResultLineItemValue::Int(n);
        } else {
            return ResultLineItemValue::Float(num.parse().unwrap());
        }
    });

    nom::branch::alt((str_parser, num_parser))(input)
}

#[cfg(test)]
mod test {
    use crate::parsing::{
        parse_item, parse_item_value, parse_result_line, ResultLineItem, ResultLineItemValue,
    };

    #[test]
    fn parse_item_value_test() {
        let s = "\"hello how are you\"rest";
        assert_eq!(
            Ok(("rest", ResultLineItemValue::Str("hello how are you"))),
            parse_item_value(s)
        );
        let s = "6124rest";
        assert_eq!(
            Ok(("rest", ResultLineItemValue::Int(6124))),
            parse_item_value(s)
        );
        let s = "149.213rest";
        assert_eq!(
            Ok(("rest", ResultLineItemValue::Float(149.213))),
            parse_item_value(s)
        );
    }

    #[test]
    fn parse_item_test() {
        let s = "my_ident=\"hello there\"rest";
        assert_eq!(
            Ok(("rest", ResultLineItem::string("my_ident", "hello there"))),
            parse_item(s)
        );
        let s = "my_ident=6124rest";
        assert_eq!(
            Ok(("rest", ResultLineItem::int("my_ident", 6124))),
            parse_item(s)
        );
        let s = "my_ident=149.213rest";
        assert_eq!(
            Ok(("rest", ResultLineItem::float("my_ident", 149.213))),
            parse_item(s)
        );
    }

    #[test]
    fn parse_result_line_test() {
        let s = "RESULT abc=\"hello there\" def=61 ghi=12.1 rest";
        let expected = vec![
            ResultLineItem::string("abc", "hello there"),
            ResultLineItem::int("def", 61),
            ResultLineItem::float("ghi", 12.1),
        ];
        assert_eq!(Ok(("rest", expected)), parse_result_line(s));
    }
}
