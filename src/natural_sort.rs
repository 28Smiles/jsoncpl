use std::{cmp::Ordering, str::Chars};
use num_bigint::BigUint;

pub fn compare(s1: &str, s2: &str) -> Ordering {
    compare_chars(s1.chars(), s2.chars()).unwrap_or(s1.cmp(s2))
}

pub fn compare_chars<'a>(c1: Chars<'a>, c2: Chars<'a>) -> Result<Ordering, ()> {
    let mut c1 = c1.peekable();
    let mut c2 = c2.peekable();

    while let (Some(x), Some(y)) = (c1.next(), c2.next()) {
        if is_digit(&x) && is_digit(&y) {
            let mut ia = BigUint::from(char_to_digit(&x)?);
            let mut ib = BigUint::from(char_to_digit(&y)?);
            while let Some(x) = c1.peek() {
                if let Ok(n) = char_to_digit(x) {
                    ia = ia * 10_u8 + n;
                    c1.next();
                } else {
                    break;
                }
            }
            while let Some(y) = c2.peek() {
                if let Ok(n) = char_to_digit(y) {
                    ib = ib * 10_u8 + n;
                    c2.next();
                } else {
                    break;
                }
            }

            match ia.cmp(&ib) {
                Ordering::Less => return Ok(Ordering::Less),
                Ordering::Equal => {},
                Ordering::Greater => return Ok(Ordering::Greater),
            }
        } else if is_digit(&x) && !is_digit(&y) {
            return Ok(Ordering::Less)
        } else if !is_digit(&x) && is_digit(&y) {
            return Ok(Ordering::Greater)
        } else if x != y {
            return Ok(x.cmp(&y));
        }
    }

    Err(())
}



fn char_to_digit(c: &char) -> Result<u32, ()> {
    match c {
        '0' => Ok(0),
        '1' => Ok(1),
        '2' => Ok(2),
        '3' => Ok(3),
        '4' => Ok(4),
        '5' => Ok(5),
        '6' => Ok(6),
        '7' => Ok(7),
        '8' => Ok(8),
        '9' => Ok(9),
        _ => Err(()),
    }
}

fn is_digit(c: &char) -> bool {
    match c {
        '0' => true,
        '1' => true,
        '2' => true,
        '3' => true,
        '4' => true,
        '5' => true,
        '6' => true,
        '7' => true,
        '8' => true,
        '9' => true,
        _ => false,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_compare() {
        assert_eq!(compare("__a__", "__1__"), Ordering::Greater);
        assert_eq!(compare("__1__", "__18__"), Ordering::Less);
        assert_eq!(compare("__1__", "__12__"), Ordering::Less);
        assert_eq!(compare("__1__", "__2__"), Ordering::Less);
        assert_eq!(compare("__10__", "__18__"), Ordering::Less);
        assert_eq!(compare("__18__", "__1__"), Ordering::Greater);
        assert_eq!(compare("__18__", "__1__"), Ordering::Greater);
        assert_eq!(compare("__18__", "__10__"), Ordering::Greater);
        assert_eq!(compare("__000912__", "__911__"), Ordering::Greater);
    }
}
