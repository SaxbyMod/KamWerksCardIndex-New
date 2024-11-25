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
    ($($expr:expr),*) => {
        $(debug!($expr);)*
    }
}

/// Helper to create hashmap.
#[macro_export]
macro_rules! hashmap {
    ($($key:expr => $value:expr,)*) => {
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
    (
        $($name:ident ($code:ident) => $link:literal,)*
        ---
        $($key:ident ($key_code:ident) => $func:ident($($func_arg:expr),*),)*
    ) => {
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
            $(
                stringify!($key_code) => {
                    let now = std::time::Instant::now();
                    let t = $func(
                        $($func_arg,)*
                        SetCode::new(stringify!($key_code)).unwrap()
                    )
                    .unwrap_or_die(&format!("Cannot process {} set", stringify!($key)))
                    .upgrade();
                    done!(
                        "Finish fetching {} set with code {} in {}",
                        $crate::Color::blue(stringify!($key)),
                        $crate::Color::yellow(stringify!($key_code)),
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

#[allow(missing_docs)]
#[macro_export]
macro_rules! frameworks {
    (global: $($gb_cmd:expr),*; $(guild($g_id:literal): $($g_cmd:expr),*;)*---$rest:block) => {
        poise::Framework::builder()
            .options(poise::FrameworkOptions {
                commands: vec![$($gb_cmd,)* $($($g_cmd,)*)*],
                event_handler: |ctx, event, fw, data| Box::pin(handler(ctx, event, fw, data)),
                ..Default::default()
            })
            .setup(|ctx, _ready, framework| {
                Box::pin(async move {
                    info!("Refreshing commands...");

                    poise::builtins::register_globally(
                        ctx.http(),
                        &[$($gb_cmd,)*]
                    )
                    .await?;

                    $(
                        let _ = poise::builtins::register_in_guild(
                            ctx.http(),
                            &[$($g_cmd,)*],
                            GuildId::from($g_id)
                        )
                        .await;
                    )*

                    done!(
                        "Finish registering {} commands",
                        framework.options().commands.len().green()
                    );

                    $rest
                })
            })
            .build()
    };
}