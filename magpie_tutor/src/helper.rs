#[macro_export]
macro_rules! info {
    ($string:literal) => {
        println!("[ {} ] {}",$crate::Color::blue("info"),format!($string))
    };
    ($string:literal,$($args:expr),*) => {
        println!("[ {} ] {}",$crate::Color::blue("info"),format!($string, $($args,)*))
    };
}
#[macro_export]
macro_rules! error {
    ($string:literal) => {
        println!("[ {} ] {}",$crate::Color::red("error"),format!($string))
    }
    ;
    ($string:literal,$($args:expr),*) => {
        println!("[ {} ] {}",$crate::Color::red("error"),format!($string, $($args,)*))
    };
}
#[macro_export]
macro_rules! done {
    ($string:literal) => {
        println!("[ {} ] {}",$crate::Color::green("done"),format!($string))
    }
    ;
    ($string:literal,$($args:expr),*) => {
        println!("[ {} ] {}",$crate::Color::green("done"),format!($string, $($args,)*))
    };
}

#[macro_export]
macro_rules! debug {
    ($string:literal) => {
        println!(
            "[ {} ] [ {}:{} ] {}",
            $crate::Color::magenta("debug"),
            file!(),
            line!(),
            $string
        )
    };
    ($expr:expr) => {
        println!(
            "[ {} ] [ {}:{} ] {} = {:?}",
            $crate::Color::magenta("debug"),
            $crate::Color::magenta(file!()),
            $crate::Color::green(&line!()),
            stringify!($expr),
            $expr,
        )
    };
}

#[macro_export]
macro_rules! hashmap {
    ($($key:expr => $value:expr,)+) => {
        {
            let mut m = std::collections::HashMap::new();

            $(m.insert($key, $value);)*

            m
        }
    };
}

#[macro_export]
macro_rules! set_map {
    ($($name:ident ($code:ident) => $link:literal),+) => {
        hashmap! {
            $(
                stringify!($code).to_owned() => {
                    let t = fetch_imf_set(
                        $link,
                        SetCode::new(stringify!($code)).unwrap()
                    )
                    .unwrap_or_die(&format!("Cannot process {} set", stringify!($name)))
                    .upgrade();

                    done!("Finish fetching {} set with code {}", stringify!($name), stringify!($code));

                    t
                },
            )*
        }
    };
}
