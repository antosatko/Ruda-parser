pub struct Parser<'a> {
    pub(crate) text: &'a str,

}

impl <'a> Parser<'a> {
    pub fn new() -> Parser<'static> {
        Parser {
            text: "",
        }
    }
}