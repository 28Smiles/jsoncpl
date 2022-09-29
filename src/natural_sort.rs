use std::{cmp::Ordering, iter::Peekable, str::Chars};

pub fn compare(s1: &str, s2: &str) -> Ordering {
    compare_chars_iters(s1.chars(), s2.chars()).unwrap_or(s1.cmp(s2))
}

pub fn compare_chars_iters<'a>(c1: Chars<'a>, c2: Chars<'a>) -> Result<Ordering, ()> {
    let mut c1 = c1.peekable();
    let mut c2 = c2.peekable();

    while let (Some(x), Some(y)) = (c1.peek(), c2.peek()) {
        if x.is_numeric() && y.is_numeric() {
            match take_numeric(&mut c1).cmp(&take_numeric(&mut c2)) {
                Ordering::Equal => (c1.next(), c2.next()),
                ref a => return Ok(*a),
            };
        } else if x.is_numeric() && !y.is_numeric() {
            return Ok(Ordering::Less)
        } else if !x.is_numeric() && y.is_numeric() {
            return Ok(Ordering::Greater)
        } else if x == y {
            c1.next();
            c2.next();
        } else {
            return Ok(x.cmp(y));
        }
    }

    Err(())
}

fn take_numeric(iter: &mut Peekable<Chars>) -> u32 {
    let mut sum = 0;

    while let Some(p) = iter.peek() {
        match p.to_string().parse::<u32>() {
            Ok(n) => {
                sum = sum * 10 + n;
                iter.next();
            }
            _ => break,
        }
    }

    sum
}

#[cfg(test)]
mod test {
    use std::cmp::Ordering;
    use crate::natural_sort::compare;

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
