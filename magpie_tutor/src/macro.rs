//! Some helper macro

/// Info print.
#[macro_export]
macro_rules! info {
    ($string:literal) => {
        println!(
            "[ {} | {} ] {}",
            $crate::Color::yellow(&chrono::Local::now().format("%H:%M:%S")),
            $crate::Color::blue("info"),
            format!($string)
        )
    };
    ($string:literal,$($args:expr),*) => {
        println!(
            "[ {} | {} ] {}",
            $crate::Color::yellow(&chrono::Local::now().format("%H:%M:%S")),
            $crate::Color::blue("info"),
            format!($string, $($args,)*)
        )
    };
}

/// Error print.
#[macro_export]
macro_rules! error {
    ($string:literal) => {
        println!(
            "[ {} | {} ] {}",
            $crate::Color::yellow(&chrono::Local::now().format("%H:%M:%S")),
            $crate::Color::red("error"),
            format!($string)
        )
    };
    ($string:literal,$($args:expr),*) => {
        println!(
            "[ {} | {} ] {}",
            $crate::Color::yellow(&chrono::Local::now().format("%H:%M:%S")),
            $crate::Color::red("error"),
            format!($string, $($args,)*)
        )
    };
}

/// Done print.
#[macro_export]
macro_rules! done {
    ($string:literal) => {
        println!(
            "[ {} | {} ] {}",
            $crate::Color::yellow(&chrono::Local::now().format("%H:%M:%S")),
            $crate::Color::green("done"),
            format!($string)
        );
    };
    ($string:literal,$($args:expr),*) => {
        {
            println!(
                "[ {} | {} ] {}",
                $crate::Color::yellow(&chrono::Local::now().format("%H:%M:%S")),
                $crate::Color::green("done"), format!($string, $($args,)*)
            );
        }
    };
}

/// Debug print.
#[macro_export]
macro_rules! debug {
    ($string:literal) => {
        println!(
            "[ {} | {} ] [ {}:{} ] {}",
            $crate::Color::yellow(&chrono::Local::now().format("%H:%M:%S")),
            $crate::Color::magenta("debug"),
            $crate::Color::magenta(file!()),
            $crate::Color::green(&line!()),
            $crate::Color::magenta($string)
        )
    };
    ($expr:expr) => {
        println!(
            "[ {} | {} ] [ {}:{} ] {} = {}",
            $crate::Color::yellow(&chrono::Local::now().format("%H:%M:%S")),
            $crate::Color::magenta("debug"),
            $crate::Color::magenta(file!()),
            $crate::Color::green(&line!()),
            $crate::Color::red(&stringify!($expr)),
            $crate::Color::magenta(&format!("{:?}", $expr)),
        )
    };
}

/// Helper to create hashmap.
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

/// Helper to create set map.
#[macro_export]
macro_rules! set_map {
    ($($name:ident ($code:ident) => $link:literal),+) => {
        hashmap! {
            $(
                stringify!($code) => {
                    let now = std::time::Instant::now();
                    let t = fetch_imf_set(
                        $link,
                        SetCode::new(stringify!($code)).unwrap()
                    )
                    .unwrap_or_die(&format!("Cannot process {} set", stringify!($name)))
                    .upgrade();

                    done!(
                        "Finish fetching {} set with code {} in {}",
                        $crate::Color::blue(stringify!($name)),
                        $crate::Color::yellow(stringify!($code)),
                        $crate::Color::green(&format!("{:.2?}", now.elapsed()))
                    );

                    t
                },
            )*
        }
    };
}

/// Helper to generate builder pattern struct
#[macro_export]
macro_rules! builder {
    (
        $(#[$attr:meta])*
        $vis:vis struct $name:ident {
            $(
                $(#[$f_attr:meta])*
                $f_vis:vis $field:ident: $f_type:ty,
            )*
        }

        $(
            impl into $ty:ty $body:block
        )*

    ) => {
        $(#[$attr])*
        #[derive(Default)]
        $vis struct $name {$(
            $(#[$f_attr])*
            $f_vis $field: $f_type,
        )*}

        impl $name {

            #[doc = concat!("Create a new [`", stringify!($name), "`] builder")]
            pub fn new() -> Self {
                Self::default()
            }


            $(
                $(#[$f_attr])*
                pub fn $field(mut self, $field: $f_type) -> Self {
                    self.$field = $field;
                    self
                }
            )*
        }
    };
}
