//! A module describing mathematical identifiers used in variable and function
//! names.

use crate::prelude::Token;

/// A mathematical identifier, for example variable or function names.
///
/// Examples of valid math identifiers: "x", "x_1", "F_g", "\overline{v}".
#[derive(Eq, PartialEq, Debug, Hash, Clone)]
pub enum MathIdentifier {
    /// Identified by a name, for example "x".
    Name(MathString),
    /// A MathIdentifier that is wrapped in an index, for example "x_1".
    ///
    /// # Examples
    /// The identifier "x_1" would be represented by `name` being
    /// `MathIdentifier::Name("x")` and `index` being
    /// `MathIdentifier::Name("1")`.
    Index {
        name: Box<MathIdentifier>,
        index: Box<MathIdentifier>,
    },
    /// A modifier applied to a MathIdentifier, for example an accent like
    /// `\overline{x}`, `\hat{x}` and `\tilde{x}`. It may also be a
    /// change in font or other text rendering, for example \mathbb{R},
    /// \mathcal{C} and \text{sin}.
    Modifier(ModifierType, Box<MathIdentifier>),
}

impl MathIdentifier {
    /// Creates a new MathIdentifier fom a vec to identify a variable and
    /// function
    /// # Deprecated
    /// MathIdentifiers are no longer represented by an array of tokens.
    #[deprecated]
    pub fn new(_tokens: Vec<Token>) -> Self {
        panic!("Use of deprecated MathIdentifier::new.");
    }
    /// Creates a new MathIdentifier from a single Token to identify a variable
    /// and a function.
    /// # Deprecated
    /// MathIdentifiers are no longer represented by tokens.
    #[deprecated]
    pub fn new_from_one(token: Token) -> Self {
        if let Token::Identifier(s) = token {
            return Self::from_single_ident(&s);
        }
        panic!();
    }
    /// Create a MathIdentifier from a single string. For example "x".
    ///
    /// Note that this method does not parse LaTeX, so passing "x_1" as a string
    /// will not return the expected MathIdentifier.
    pub fn from_single_ident(s: &str) -> Self {
        Self::Name(MathString::from_letters(
            s.as_bytes()
                .iter()
                .map(|byte| MathLetter::Ascii(*byte))
                .collect(),
        ))
    }

    pub fn from_single_greek(letter: GreekLetter) -> Self {
        Self::Name(MathString::from_letters(vec![MathLetter::Greek(letter)]))
    }
}

#[macro_export]
macro_rules! enum_with_name {
    ($name:ident { $($variant:ident),* $(,)? }) => {
        #[derive(Eq, PartialEq, Debug, Hash, Clone)]
        pub enum $name {
            $($variant),*
        }

        impl $name {
            fn name(&self) -> &str {
                match self {
                    $(Self::$variant => stringify!($variant)),*
                }
            }
        }
    }
}

enum_with_name!(ModifierType {
    Overline,
    Hat,
    Tilde,
    Bar,
    Breve,
    Check,
    Dot,
    Ddot,
    Vec,
    Mathring,
    Text,
    Mathbb,
    Mathcal,
});

#[derive(Eq, PartialEq, Debug, Hash, Clone)]
pub struct MathString {
    vec: Vec<MathLetter>,
}

impl MathString {
    pub fn from_letters(vec: Vec<MathLetter>) -> Self {
        Self { vec }
    }
}

#[derive(Eq, PartialEq, Debug, Hash, Clone)]
pub enum MathLetter {
    Ascii(u8),
    Greek(GreekLetter),
    Other(OtherSymbol),
}

enum_with_name!(GreekLetter {
    UppercaseGamma,
    UppercaseDelta,
    UppercaseTheta,
    UppercaseLambda,
    UppercaseXi,
    UppercasePi,
    UppercaseSigma,
    UppercaseUpsilon,
    UppercasePsi,
    UppercaseOmega,
    VarUppercaseGamma,
    VarUppercaseDelta,
    VarUppercaseTheta,
    VarUppercaseLambda,
    VarUppercaseXi,
    VarUppercasePi,
    VarUppercaseSigma,
    VarUppercaseUpsilon,
    VarUppercasePhi,
    VarUppercasePsi,
    VarUppercaseOmega,
    LowercaseAlpha,
    LowercaseBeta,
    LowercaseGamma,
    LowercaseDelta,
    LowercaseEpsilon,
    LowercaseZeta,
    LowercaseEta,
    LowercaseTheta,
    LowercaseIota,
    LowercaseKappa,
    LowercaseLambda,
    LowercaseMu,
    LowercaseNu,
    LowercaseXi,
    LowercaseOmicron,
    LowercasePi,
    LowercaseRho,
    LowercaseSigma,
    LowercaseTau,
    LowercaseUpsilon,
    LowercasePhi,
    LowercaseChi,
    LowercasePsi,
    LowercaseOmega,
    VarLowercaseEpsilon,
    VarLowercaseKappa,
    VarLowercaseTheta,
    VheLowercaseTasym,
    VarLowercasePi,
    VarLowercaseRho,
    VarLowercaseSigma,
    VarLowercasePhi,
});

impl GreekLetter {
    pub fn latex_code(&self) -> String {
        let mut name = self.name();
        let var = name.starts_with("Var");
        if var {
            name = &name[3..];
        }
        let lowercase = match &name[0..9] {
            "Lowecase" => true,
            "Uppercase" => false,
            _ => panic!("Wrong enum variant name."),
        };
        let mut res = name[10..].to_string();
        if lowercase {
            let mut chars = res.chars();
            res = format!(
                "{}{}",
                chars.next().unwrap().to_ascii_lowercase(),
                chars.as_str()
            );
        }
        if var {
            res = format!("var{}", res);
        }

        res
    }
}

enum_with_name!(OtherSymbol { Sin, Cos, Tan, Ln });

impl OtherSymbol {
    pub fn latex_code(&self) -> String {
        self.name().to_lowercase()
    }
}

#[cfg(test)]
mod tests {
    use crate::identifier::GreekLetter;

    #[test]
    fn greek_letters_latex() {
        assert_eq!(GreekLetter::LowercaseAlpha.latex_code(), "alpha");
        assert_eq!(GreekLetter::UppercasePi.latex_code(), "Pi");
        assert_eq!(GreekLetter::VarUppercaseDelta.latex_code(), "varDelta");
        assert_eq!(GreekLetter::VarLowercasePi.latex_code(), "varpi");
    }
}
