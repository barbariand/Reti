
use super::MathExprKey;

// Represents a square root or nth root
#[derive(Debug, Clone)]
pub struct Root {
    pub degree: Option<MathExprKey>, // None for square roots, Some for nth roots
    pub radicand: MathExprKey,
}
/*
impl Root {
    pub async fn parse(parser: &mut Parser) -> Result<Root, ParseError> {
        // \sqrt{123}x
        //       <mul>
        //     /       \
        // \sqrt{123}    x
        // let arr:Vec<MathExpr>=[123,sqrt(&1),x,mul(&2,&3)]

        // x=2
        // [123,sqrt(&1),x,mul(&2,&3)]
        // 123 vad värde? 123
        // sqrt(&1) vad värde? vänta jag ska kolla med &1, hämtar 123 och tar5 sqrt mitt värde är 11
        // x vad värde? 2
        // mul(&2,&3) vad värde? vänta jag ska kolla med de andra? hämtar 123 och 2 mitt värde är 246
        // jag är klar detta betyder att värde av allt är den sista som vi kollade

        // \sqrt{123x}x
        //            <mul>
        //          /       \
        //      \sqrt        x
        //        |
        //      <mul>
        //        |
        //     /     \
        //    123    x
        //[123,x,mul(&1,&2),]

        //
        // \sqrt{123}
        // \sqrt[123]{123}
        //
        //self.state.push(MathExpr)->MathExperKey
        let mut degree: Option<MathExprKey> = None;
        if parser.reader.peek().await == Token::LeftBracket {
            // We found a square bracket containing the square root degree.

            parser.reader.skip().await; // skip [
                                        // Read expression for degree
            let degree_expr = parser.expr().await?;
            parser.expect(Token::RightBracket).await?; // expect ]
            degree = Some(parser.get_key(degree_expr));
        }

        parser.expect(Token::LeftCurlyBracket).await?;
        let radicand_expr = parser.expr().await?;
        parser.expect(Token::RightCurlyBracket).await?;
        let radicand = parser.get_key(radicand_expr);
        Ok(Root { degree, radicand })
    }
}
*/
