pub use anstream::println as aprintln;

/// Tokyo Night color palette
#[allow(dead_code)]
pub mod colors {
    pub const RESET: &str = "\x1b[0m";

    pub const TKN_RED: &str = "\x1b[38;2;247;118;142m"; // #f7768e
    pub const TKN_GREEN: &str = "\x1b[38;2;158;206;106m"; // #9ece6a
    pub const TKN_YELLOW: &str = "\x1b[38;2;224;175;104m"; // #e0af68
    pub const TKN_BLUE: &str = "\x1b[38;2;122;162;247m"; // #7aa2f7
    pub const TKN_MAGENTA: &str = "\x1b[38;2;187;154;247m"; // #bb9af7
    pub const TKN_CYAN: &str = "\x1b[38;2;125;207;255m"; // #7dcfff
}

pub struct ColorPrinter;

impl ColorPrinter {
    pub fn green(text: &str) -> String {
        format!("{}{}{}", colors::TKN_GREEN, text, colors::RESET)
    }

    pub fn red(text: &str) -> String {
        format!("{}{}{}", colors::TKN_RED, text, colors::RESET)
    }

    pub fn yellow(text: &str) -> String {
        format!("{}{}{}", colors::TKN_YELLOW, text, colors::RESET)
    }

    pub fn blue(text: &str) -> String {
        format!("{}{}{}", colors::TKN_BLUE, text, colors::RESET)
    }

    #[allow(dead_code)]
    pub fn magenta(text: &str) -> String {
        format!("{}{}{}", colors::TKN_MAGENTA, text, colors::RESET)
    }

    pub fn cyan(text: &str) -> String {
        format!("{}{}{}", colors::TKN_CYAN, text, colors::RESET)
    }
}

pub fn p_g(text: &str) -> String {
    ColorPrinter::green(text)
}

pub fn p_r(text: &str) -> String {
    ColorPrinter::red(text)
}

pub fn p_y(text: &str) -> String {
    ColorPrinter::yellow(text)
}

pub fn p_b(text: &str) -> String {
    ColorPrinter::blue(text)
}

#[allow(dead_code)]
pub fn p_m(text: &str) -> String {
    ColorPrinter::magenta(text)
}

pub fn p_c(text: &str) -> String {
    ColorPrinter::cyan(text)
}
