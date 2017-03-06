extern crate wikipedia;
extern crate regex;
extern crate time;

fn try_parse_dmy(s: &str) -> Result<(time::Tm, usize), ()> {
    let re = regex::Regex::new("(\\d{1,2}\\s+[A-Za-z]+\\s+\\d{1,4})").unwrap();
    re.captures_iter(&s).next().and_then(|c| {
        let s = c.get(1).unwrap();
        time::strptime(s.as_str(), "%d %B %Y").ok().map(|x| (x, s.end()))
    }).ok_or(())
}

fn try_parse_mdy(s: &str) -> Result<(time::Tm, usize), ()> {
    let re = regex::Regex::new("([A-Za-z]+\\s+\\d{1,2},\\s+\\d{1,4})").unwrap();
    re.captures_iter(&s).next().and_then(|c| {
        let s = c.get(1).unwrap();
        time::strptime(s.as_str(), "%B %d, %Y").ok().map(|x| (x, s.end()))
    }).ok_or(())
}

fn try_parse_one(s: &str) -> Result<(time::Tm, usize), ()> {
    try_parse_dmy(s).or_else(|_| try_parse_mdy(s))
}

fn try_parse_dates(s: &str) -> Result<(time::Tm, Option<time::Tm>), ()> {
    if let Ok((d1, index)) = try_parse_one(s) {
        Ok((d1, try_parse_one(&s[index..]).map(|x| x.0).ok()))
    } else {
        Err(())
    }
}

fn is_alive(name: &str) -> Result<Option<(String, Option<time::Tm>)>, wikipedia::Error> {
    let re = regex::Regex::new("\\((.*?)\\)").unwrap();
    let wiki = wikipedia::Wikipedia::<wikipedia::http::hyper::Client>::default();
    let page = wiki.page_from_title(name.to_owned());
    let content = page.get_summary()?;
    if let Some(c) = re.captures_iter(&content).next() {
        match try_parse_dates(c.get(1).unwrap().as_str()) {
            Ok((_, death)) => Ok(Some((page.get_title()?, death))),
            _ => Ok(None),
        }
    } else {
        Ok(None)
    }
}

fn main() {
    let mut args = std::env::args();
    args.next();
    for arg in args {
        match is_alive(&arg) {
            Ok(Some((name, None))) => println!("{} is alive", name),
            Ok(Some((name, Some(death)))) => println!("{} is not alive ({})",
                name,
                death.strftime("%Y-%m-%d").unwrap()
            ),
            Ok(None) => println!("Cannot find information about {}", arg),
            _ => println!("Error checking {}", arg),
        }
    }
}
