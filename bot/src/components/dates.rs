pub struct DatePrompt<'a> {
    pub question: &'a str,
    pub choices: Vec<(&'a str, i8)>,
}
impl DatePrompt {
    pub fn new(question: &str, choices: Vec<(&'a str, i8)>) -> Self {
        Self { question, choices }
    }
}

const QUESTIONS: Vec<DatePrompt<'_>> =
    vec![DatePrompt::new("lol idk", vec![("Yes", +10), ("No", -10)])];
