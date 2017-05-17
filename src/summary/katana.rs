use regex::{Regex, Captures};

pub fn cut(origin_text: &String) -> Vec<String> {
    // Remove composite abbreviations.
    let composite = Regex::new(r"(?P<comp>et al\.)(?:\.)").unwrap();
    let mut text = (*composite.replace_all(&origin_text.to_string(), "$comp&;&")).to_string();

    // // Remove suspension points.
    let suspension: Regex = Regex::new(r"\.{3}").unwrap();
    text = (*suspension.replace_all(&text.to_string(), "&&&.")).to_string();

    // // Remove floating point numbers.
    let float_point_reg: Regex = Regex::new(r"(?P<number>[0-9]+)\.(?P<decimal>[0-9]+)").unwrap();
    text = (*float_point_reg.replace_all(&text.to_string(), "$number&@&$decimal")).to_string();

    // Handle floats without leading zero.
    let floats_without_zeros = Regex::new(r"\s\.(?P<nums>[0-9]+)").unwrap();
    text = (*floats_without_zeros.replace_all(&text.to_string(), " &#&$nums")).to_string();

    // Remove abbreviations.
    let abbrev = Regex::new(r"(?:[A-Za-z]\.){2,}").unwrap();
    text = (*abbrev.replace_all(&text.to_string(), |caps: &Captures| {
        caps.iter()
            .map(|c| c.unwrap().as_str().to_string().replace(".", "&-&"))
            .collect()
    }))
            .to_string();

    // Remove initials.
    let initials = Regex::new(r"(?P<init>[A-Z])(?P<point>\.)").unwrap();
    text = (*initials.replace_all(&text.to_string(), "$init&_&")).to_string();

    // Remove titles.
    let titles = Regex::new(r"(?P<title>[A-Z][a-z]{1,3})(\.)").unwrap();
    text = (*titles.replace_all(&text.to_string(), "$title&*&")).to_string();

    // Unstick sentences from each other.
    let unstick = Regex::new(r##"(?P<left>[^.?!]\.|!|\?)(?P<right>[^\s"'])"##).unwrap();
    text = (*unstick.replace_all(&text.to_string(), "$left $right")).to_string();

    // Remove sentence enders before parens
    let before_parens = Regex::new(r##"(?P<bef>[.?!])\s?\)"##).unwrap();
    text = (*before_parens.replace_all(&text.to_string(), "&==&$bef")).to_string();

    // Remove sentence enders next to quotes.
    let quote_one = Regex::new(r##"'(?P<quote>[.?!])\s?""##).unwrap();
    text = (*quote_one.replace_all(&text.to_string(), "&^&$quote")).to_string();

    let quote_two = Regex::new(r##"'(?P<quote>[.?!])\s?”"##).unwrap();
    text = (*quote_two.replace_all(&text.to_string(), "&**&$quote")).to_string();

    let quote_three = Regex::new(r##"(?P<quote>[.?!])\s?”"##).unwrap();
    text = (*quote_three.replace_all(&text.to_string(), "&=&$quote")).to_string();

    let quote_four = Regex::new(r##"(?P<quote>[.?!])\s?'""##).unwrap();
    text = (*quote_four.replace_all(&text.to_string(), "&,&$quote")).to_string();

    let quote_five = Regex::new(r##"(?P<quote>[.?!])\s?'"##).unwrap();
    text = (*quote_five.replace_all(&text.to_string(), "&##&$quote")).to_string();

    let quote_six = Regex::new(r##"(?P<quote>[.?!])\s?""##).unwrap();
    text = (*quote_six.replace_all(&text.to_string(), "&$&$quote")).to_string();

    // Split on any sentence ender.
    let s: Vec<&str> = text.split("!").collect();
    let s_last = s.len() - 1;
    let mut s_one: Vec<String> = s[0..s_last]
        .iter()
        .map(|s| String::from(*s) + "!")
        .collect();
    s_one.push(String::from(s[s_last]));

    let mut s_two: Vec<String> = Vec::new();
    for sen in s_one.iter() {
        let ss: Vec<&str> = sen.split("?").collect();
        let mut tmp_vec: Vec<String> = ss[0..ss.len() - 1]
            .iter()
            .map(|s| String::from(*s) + "?")
            .collect();
        s_two.append(&mut tmp_vec);
        s_two.push(String::from(ss[ss.len() - 1]));
    }

    let mut final_vec: Vec<String> = Vec::new();
    for sen in s_two.iter() {
        let ss: Vec<&str> = sen.split(".").collect();
        let mut tmp_vec: Vec<String> = ss[0..ss.len() - 1]
            .iter()
            .map(|s| String::from(*s) + ".")
            .collect();
        final_vec.append(&mut tmp_vec);
        final_vec.push(String::from(ss[ss.len() - 1]));
    }

    // Repair the damage we've done.

    // Prepare the Regexes for quote repair
    let paren_repair = Regex::new(r"&==&(?P<p>[.!?])").unwrap();

    let quote_one_repair = Regex::new(r"&\^&(?P<p>[.!?])").unwrap();
    let quote_two_repair = Regex::new(r"&\*\*&(?P<p>[.!?])").unwrap();
    let quote_three_repair = Regex::new(r"&=&(?P<p>[.!?])").unwrap();
    let quote_four_repair = Regex::new(r#"&,&(?P<p>[.!?])"#).unwrap();
    let quote_five_repair = Regex::new(r"&##&(?P<p>[.!?])").unwrap();
    let quote_six_repair = Regex::new(r"&\$&(?P<p>[.!?])").unwrap();

    let results: Vec<String> = final_vec.iter()
        .map(|s| {
            // Skip whitespace zones.
            s.trim()
                // Repair composite abbreviations.
                .replace("&;&", ".")
                // Repair suspension points.
                .replace("&&&", "..")
                // Repair Floats.
                .replace("&@&", ".")
                // Repair floats without leading zeros
                .replace("&#&", ".")
                // Repair abbreviations.
                .replace("&-&", ".")
                // Repair intials.
                .replace("&_&", ".")
                // Repair titles.
                .replace("&*&", ".")
        })
    .map(|s| {
        (*paren_repair.replace_all(&s.to_string(), r"$1)")).to_string()
    })
    // Repair quotes with sentence enders.
    .map(|s| {
        (*quote_one_repair.replace_all(&s.to_string(), r#"'$p""#)).to_string()
    })
    .map(|s| {
        (*quote_two_repair.replace_all(&s.to_string(), r#"'$p”"#)).to_string()
    })
    .map(|s| {
        (*quote_three_repair.replace_all(&s.to_string(), r#"$p”"#)).to_string()
    })
    .map(|s| {
        (*quote_four_repair.replace_all(&s.to_string(), r#"'""#)).to_string()
    })
    .map(|s| {
        (*quote_five_repair.replace_all(&s.to_string(), r#"$p'"#)).to_string()
    })
    .map(|s| {
        (*quote_six_repair.replace_all(&s.to_string(), r#"$p""#)).to_string()
    })
    .filter_map(|s| {
        if s.len() > 1 {
            Some(s.to_string())
        } else {
            None
        }
    })
    .collect();

    results.to_owned()
}
