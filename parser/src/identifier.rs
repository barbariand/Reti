//! A module describing mathematical identifiers used in variable and function
//! names.

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
        /// The base part of the identifier.
        name: Box<MathIdentifier>,
        /// The index identifier.
        index: Box<MathIdentifier>,
    },
    /// A modifier applied to a MathIdentifier, for example an accent like
    /// `\overline{x}`, `\hat{x}` and `\tilde{x}`. It may also be a
    /// change in font or other text rendering, for example \mathbb{R},
    /// \mathcal{C} and \text{sin}.
    Modifier(ModifierType, Box<MathIdentifier>),
}

impl MathIdentifier {
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

    /// Create a MathIdentifier from a single greek letter. For example
    /// "\lambda".
    pub fn from_single_greek(letter: GreekLetter) -> Self {
        Self::Name(MathString::from_letters(vec![MathLetter::Greek(letter)]))
    }

    /// Create a MathIdentifier from a single symbol. For example
    /// "\ln".
    pub fn from_single_symbol(symbol: OtherSymbol) -> Self {
        Self::Name(MathString::from_letters(vec![MathLetter::Other(symbol)]))
    }
}

/// A macro for creating an enum with LaTeX.
///
/// # Example
/// ```
/// enum_with_latex!(ModifierType {
/// Overline => "overline",
/// Hat => "hat",
/// });
/// ```
#[macro_export]
macro_rules! enum_with_latex {
    ($name:ident { $($variant:ident => $latex:expr),* $(,)? }) => {
        /// Generated enum. TODO Cindy fix this.
        #[derive(Eq, PartialEq, Debug, Hash, Clone)]
        pub enum $name {
            $(
                /// Generated enum variant.
                /// LaTeX: $latex
                $variant
            ),*
        }

        impl $name {
            /// Get the LaTeX code for this enum variant.
            pub fn latex_code(&self) -> &'static str {
                match self {
                    $(Self::$variant => $latex),*
                }
            }

            /// Get enum variant by LaTeX.
            pub fn from_latex(latex: &str) -> Option<Self> {
                match latex {
                    $($latex => Some(Self::$variant)),*,
                    _ => None,
                }
            }
        }
    }
}

enum_with_latex!(ModifierType {
    Overline => "overline",
    Hat => "hat",
    Tilde => "tilde",
    Bar => "bar",
    Breve => "breve",
    Check => "check",
    Dot => "dot",
    Ddot => "ddot",
    Vec => "vec",
    Mathring => "mathring",
    Text => "text",
    Mathbb => "mathbb",
    Mathcal => "mathcal",
});

/// A string of mathematical letters that may consist of greek letters and other
/// math symbols.
#[derive(Eq, PartialEq, Debug, Hash, Clone)]
pub struct MathString {
    vec: Vec<MathLetter>,
}

impl MathString {
    /// Create a MathString from letters.
    pub fn from_letters(vec: Vec<MathLetter>) -> Self {
        Self { vec }
    }
}

/// A mathematical letter. Either ascii (letter from the English alphabet), a
/// greek letter, or another mathematical symbol.
#[derive(Eq, PartialEq, Debug, Hash, Clone)]
pub enum MathLetter {
    /// A letter in the English alphabet, for example "a", "x".
    ///
    /// Represented by the ascii code.
    Ascii(u8),
    /// A greek letter.
    Greek(GreekLetter),
    /// A mathematical symbol.
    Other(OtherSymbol),
}

impl MathLetter {
    /// Get a MathLetter from LaTeX code.
    pub fn from_latex(latex: &str) -> Option<Self> {
        if let Some(letter) = GreekLetter::from_latex(latex) {
            return Some(Self::Greek(letter));
        }
        if let Some(symbol) = OtherSymbol::from_latex(latex) {
            return Some(Self::Other(symbol));
        }
        None
    }
}

enum_with_latex!(GreekLetter {
    UppercaseGamma => "Gamma",
    UppercaseDelta => "Delta",
    UppercaseTheta => "Theta",
    UppercaseLambda => "Lambda",
    UppercaseXi => "Xi",
    UppercasePi => "Pi",
    UppercaseSigma => "Sigma",
    UppercaseUpsilon => "Upsilon",
    UppercasePsi => "Psi",
    UppercaseOmega => "Omega",
    VarUppercaseGamma => "varGamma",
    VarUppercaseDelta => "varDelta",
    VarUppercaseTheta => "varTheta",
    VarUppercaseLambda => "varLambda",
    VarUppercaseXi => "varXi",
    VarUppercasePi => "varPi",
    VarUppercaseSigma => "varSigma",
    VarUppercaseUpsilon => "varUpsilon",
    VarUppercasePhi => "varPhi",
    VarUppercasePsi => "varPsi",
    VarUppercaseOmega => "varOmega",
    LowercaseAlpha => "alpha",
    LowercaseBeta => "beta",
    LowercaseGamma => "gamma",
    LowercaseDelta => "delta",
    LowercaseEpsilon => "epsilon",
    LowercaseZeta => "zeta",
    LowercaseEta => "eta",
    LowercaseTheta => "theta",
    LowercaseIota => "iota",
    LowercaseKappa => "kappa",
    LowercaseLambda => "lambda",
    LowercaseMu => "mu",
    LowercaseNu => "nu",
    LowercaseXi => "xi",
    LowercaseOmicron => "omicron",
    LowercasePi => "pi",
    LowercaseRho => "rho",
    LowercaseSigma => "sigma",
    LowercaseTau => "tau",
    LowercaseUpsilon => "upsilon",
    LowercasePhi => "phi",
    LowercaseChi => "chi",
    LowercasePsi => "psi",
    LowercaseOmega => "omega",
    VarLowercaseEpsilon => "varepsilon",
    VarLowercaseKappa => "varkappa",
    VarLowercaseTheta => "vartheta",
    VarLowercaseTasym => "vartasym",
    VarLowercasePi => "varpi",
    VarLowercaseRho => "varrho",
    VarLowercaseSigma => "varsigma",
    VarLowercasePhi => "varphi",
});

enum_with_latex!(OtherSymbol {
    Sin => "sin",
    Cos => "cos",
    Tan => "tan",
    Ln => "ln",
});

#[cfg(test)]
mod tests {
    use crate::identifier::GreekLetter;

    #[test]
    fn greek_letters_to_latex() {
        assert_eq!(GreekLetter::LowercaseAlpha.latex_code(), "alpha");
        assert_eq!(GreekLetter::UppercasePi.latex_code(), "Pi");
        assert_eq!(GreekLetter::VarUppercaseDelta.latex_code(), "varDelta");
        assert_eq!(GreekLetter::VarLowercasePi.latex_code(), "varpi");
    }

    fn from_latex_test(latex: &'static str, letter: GreekLetter) {
        assert_eq!(GreekLetter::from_latex(latex).unwrap(), letter);
    }

    #[test]
    fn greek_letters_from_latex() {
        from_latex_test("alpha", GreekLetter::LowercaseAlpha);
        from_latex_test("Pi", GreekLetter::UppercasePi);
        from_latex_test("varDelta", GreekLetter::VarUppercaseDelta);
        from_latex_test("varpi", GreekLetter::VarLowercasePi);
    }
}
