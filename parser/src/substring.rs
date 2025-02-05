use anyhow::Result;
use std::{collections::HashMap, vec};

use crate::{api::RegexId, grammar_builder::RegexBuilder};

#[derive(Debug)]
struct State<'a> {
    len: usize,
    link: Option<usize>,
    next: HashMap<&'a str, usize>,
}

/// For details see https://en.wikipedia.org/wiki/Suffix_automaton.
/// Implementation is based on https://cp-algorithms.com/string/suffix-automaton.html
struct SuffixAutomaton<'a> {
    states: Vec<State<'a>>,
    last: usize,
}

impl<'a> SuffixAutomaton<'a> {
    fn new() -> Self {
        let init_state = State {
            len: 0,
            link: None,
            next: HashMap::new(),
        };
        SuffixAutomaton {
            states: vec![init_state],
            last: 0,
        }
    }

    fn from_string(chunks: Vec<&'a str>) -> Self {
        let mut sa = SuffixAutomaton::new();
        for s in chunks.into_iter() {
            sa.extend(s);
        }
        sa
    }

    fn extend(&mut self, s: &'a str) {
        let cur_index = self.states.len();
        self.states.push(State {
            len: self.states[self.last].len + 1,
            link: None,
            next: HashMap::new(),
        });

        let mut p = Some(self.last);
        while let Some(pp) = p {
            if self.states[pp].next.contains_key(&s) {
                break;
            }
            self.states[pp].next.insert(s, cur_index);
            p = self.states[pp].link;
        }

        if let Some(pp) = p {
            let q = self.states[pp].next[&s];
            if self.states[pp].len + 1 == self.states[q].len {
                self.states[cur_index].link = Some(q);
            } else {
                let clone_index = self.states.len();
                self.states.push(State {
                    len: self.states[pp].len + 1,
                    link: self.states[q].link,
                    next: self.states[q].next.clone(),
                });
                while let Some(ppp) = p {
                    if self.states[ppp].next[&s] == q {
                        self.states[ppp].next.insert(s, clone_index);
                    } else {
                        break;
                    }
                    p = self.states[ppp].link;
                }
                self.states[q].link = Some(clone_index);
                self.states[cur_index].link = Some(clone_index);
            }
        } else {
            self.states[cur_index].link = Some(0);
        }
        self.last = cur_index;
    }
}

pub fn substring(builder: &mut RegexBuilder, chunks: Vec<&str>) -> Result<RegexId> {
    let sa = SuffixAutomaton::from_string(chunks);
    let mut state_stack = vec![0];
    let mut node_cache: HashMap<usize, RegexId> = HashMap::new();

    let empty = builder.literal("".to_string());

    while let Some(state_index) = state_stack.last() {
        let state = &sa.states[*state_index];
        if node_cache.contains_key(state_index) {
            state_stack.pop();
            continue;
        }

        if state.next.is_empty() {
            node_cache.insert(*state_index, empty);
            state_stack.pop();
            continue;
        }

        let unprocessed_children = state
            .next
            .values()
            .filter(|child_index| !node_cache.contains_key(child_index))
            .collect::<Vec<_>>();
        if !unprocessed_children.is_empty() {
            state_stack.extend(unprocessed_children);
            continue;
        }

        let mut options = state
            .next
            .keys()
            .map(|c| {
                let lit = builder.literal(c.to_string());
                builder.concat(vec![lit, node_cache[&state.next[c]]])
            })
            .collect::<Vec<_>>();
        options.push(empty);
        let expr = builder.or(options);
        node_cache.insert(*state_index, expr);
        state_stack.pop();
    }
    Ok(node_cache[&0])
}

pub fn chunk_into_chars(input: &str) -> Vec<&str> {
    let mut chunks = vec![];
    let mut char_indices = input.char_indices().peekable();

    while let Some((start, _)) = char_indices.next() {
        let end = match char_indices.peek() {
            Some(&(next_index, _)) => next_index,
            None => input.len(),
        };
        chunks.push(&input[start..end]);
    }

    chunks
}

#[derive(PartialEq)]
enum TokenType {
    Whitespace,
    Word,
    Other,
}

fn classify(ch: char) -> TokenType {
    if ch.is_whitespace() {
        TokenType::Whitespace
    } else if ch.is_alphanumeric() || ch == '_' {
        TokenType::Word
    } else {
        TokenType::Other
    }
}

pub fn chunk_into_words(input: &str) -> Vec<&str> {
    if input.is_empty() {
        return Vec::new();
    }
    let mut chunks = Vec::new();
    let mut start = 0;
    let mut current_type = classify(input.chars().next().unwrap());
    for (i, ch) in input.char_indices() {
        let token_type = classify(ch);
        if token_type != current_type {
            chunks.push(&input[start..i]);
            start = i;
            current_type = token_type;
        }
    }
    chunks.push(&input[start..]);
    chunks
}

#[cfg(test)]
mod test {
    use derivre::Regex;

    use crate::{
        api::{ParserLimits, RegexId},
        earley::regex_nodes_to_derivre,
        grammar_builder::RegexBuilder,
    };

    use super::{chunk_into_chars, chunk_into_words, substring};

    fn to_regex(mut builder: RegexBuilder, expr: RegexId) -> Regex {
        let limits = ParserLimits::default();
        let mut d_builder = derivre::RegexBuilder::new();
        let r = regex_nodes_to_derivre(&mut d_builder, &limits, builder.finalize()).unwrap();
        let eref = r[expr.0];
        d_builder.to_regex(eref)
    }

    #[test]
    fn test_tokenize_chars() {
        let input = "The quick brown fox jumps over the lazy dog.";
        let tokens = chunk_into_chars(input);
        assert_eq!(input, tokens.join(""));
        assert_eq!(
            tokens,
            vec![
                "T", "h", "e", " ", "q", "u", "i", "c", "k", " ", "b", "r", "o", "w", "n", " ",
                "f", "o", "x", " ", "j", "u", "m", "p", "s", " ", "o", "v", "e", "r", " ", "t",
                "h", "e", " ", "l", "a", "z", "y", " ", "d", "o", "g", "."
            ]
        );
    }

    #[test]
    fn test_tokenize_chars_unicode() {
        let input = "빠른 갈색 여우가 게으른 개를 뛰어넘었다.";
        let tokens = chunk_into_chars(input);
        assert_eq!(input, tokens.join(""));
        assert_eq!(
            tokens,
            vec![
                "빠", "른", " ", "갈", "색", " ", "여", "우", "가", " ", "게", "으", "른", " ",
                "개", "를", " ", "뛰", "어", "넘", "었", "다", "."
            ]
        );
    }

    #[test]
    fn test_tokenize_words() {
        let input = "The quick brown fox jumps over the lazy dog.";
        let tokens = chunk_into_words(input);
        assert_eq!(input, tokens.join(""));
        assert_eq!(
            tokens,
            vec![
                "The", " ", "quick", " ", "brown", " ", "fox", " ", "jumps", " ", "over", " ",
                "the", " ", "lazy", " ", "dog", "."
            ]
        );
    }

    #[test]
    fn test_tokenize_words_unicode() {
        let input = "빠른 갈색 여우가 게으른 개를 뛰어넘었다.";
        let tokens = chunk_into_words(input);
        assert_eq!(input, tokens.join(""));
        assert_eq!(
            tokens,
            vec![
                "빠른",
                " ",
                "갈색",
                " ",
                "여우가",
                " ",
                "게으른",
                " ",
                "개를",
                " ",
                "뛰어넘었다",
                "."
            ]
        );
    }

    #[test]
    fn test_substring_chars() {
        let mut builder = RegexBuilder::new();
        let expr = substring(
            &mut builder,
            chunk_into_chars("The quick brown fox jumps over the lazy dog."),
        )
        .unwrap();
        let regex = to_regex(builder, expr);
        assert_eq!(
            regex
                .clone()
                .is_match("The quick brown fox jumps over the lazy dog."),
            true
        );
        assert_eq!(regex.clone().is_match("The quick brown fox"), true);
        assert_eq!(regex.clone().is_match("he quick brow"), true);
        assert_eq!(regex.clone().is_match("fox jump"), true);
        assert_eq!(regex.clone().is_match("dog."), true);
        assert_eq!(regex.clone().is_match("brown fx"), false);
    }

    #[test]
    fn test_substring_chars_unicode() {
        let mut builder = RegexBuilder::new();
        let expr = substring(
            &mut builder,
            chunk_into_chars("빠른 갈색 여우가 게으른 개를 뛰어넘었다."),
        )
        .unwrap();
        let regex = to_regex(builder, expr);
        assert_eq!(
            regex
                .clone()
                .is_match("빠른 갈색 여우가 게으른 개를 뛰어넘었다."),
            true
        );
        assert_eq!(regex.clone().is_match("빠른 갈색 여우가 게으른"), true);
        assert_eq!(regex.clone().is_match("른 갈색 여우"), true);
        assert_eq!(regex.clone().is_match("여우가 게으"), true);
        assert_eq!(regex.clone().is_match("뛰어넘었다."), true);
        assert_eq!(regex.clone().is_match("갈색 여가"), false);
    }

    #[test]
    fn test_substring_words() {
        let mut builder = RegexBuilder::new();
        let expr = substring(
            &mut builder,
            chunk_into_words("The quick brown fox jumps over the lazy dog."),
        )
        .unwrap();
        let regex = to_regex(builder, expr);
        assert_eq!(
            regex
                .clone()
                .is_match("The quick brown fox jumps over the lazy dog."),
            true
        );
        assert_eq!(regex.clone().is_match("The quick brown fox"), true);
        assert_eq!(regex.clone().is_match("he quick brow"), false);
        assert_eq!(regex.clone().is_match("fox jump"), false);
        assert_eq!(regex.clone().is_match("dog."), true);
        assert_eq!(regex.clone().is_match("brown fx"), false);
    }

    #[test]
    fn test_substring_words_unicode() {
        let mut builder = RegexBuilder::new();
        let expr = substring(
            &mut builder,
            chunk_into_words("빠른 갈색 여우가 게으른 개를 뛰어넘었다."),
        )
        .unwrap();
        let regex = to_regex(builder, expr);
        assert_eq!(
            regex
                .clone()
                .is_match("빠른 갈색 여우가 게으른 개를 뛰어넘었다."),
            true
        );
        assert_eq!(regex.clone().is_match("빠른 갈색 여우가 게으른"), true);
        assert_eq!(regex.clone().is_match("른 갈색 여우"), false);
        assert_eq!(regex.clone().is_match("여우가 게으"), false);
        assert_eq!(regex.clone().is_match("뛰어넘었다."), true);
        assert_eq!(regex.clone().is_match("갈색 여가"), false);
    }
}
