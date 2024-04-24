use crate::{
    toktree::{Recognizer, SpecialToken, TokTrie},
    AiciCtrl, MidProcessArg, MidProcessResult,
};
use std::fmt::Debug;

pub struct AiciRecognizer<R: Recognizer> {
    pub trie: TokTrie,
    pub rec: R,
}

impl<R: Recognizer> AiciRecognizer<R> {
    pub fn from_recognizer(rec: R) -> Self {
        AiciRecognizer {
            trie: TokTrie::from_host(),
            rec,
        }
    }
}

impl<R: Recognizer + Clone> AiciCtrl for AiciRecognizer<R> {
    fn mid_process(&mut self, arg: MidProcessArg) -> MidProcessResult {
        if arg.has_eos() {
            return MidProcessResult::stop();
        }
        self.trie.append_tokens(&mut self.rec, &arg.tokens);
        let mut set = self.trie.alloc_token_set();
        self.trie.compute_bias(&mut self.rec, &mut set);
        MidProcessResult::sample(set)
    }
}

pub trait FunctionalRecognizer<S: Copy> {
    /// Initial state
    fn initial(&self) -> S;
    /// Extend the recognizer with given byte if allowed.
    fn try_append(&self, state: S, byte: u8) -> Option<S>;
    /// Check if given special token is allowed in given state.
    fn special_allowed(&self, state: S, tok: SpecialToken) -> bool;
}

#[derive(Clone)]
pub struct StackRecognizer<S: Copy, R: FunctionalRecognizer<S>> {
    rec: R,
    stack: Vec<S>,
    stack_ptr: usize,
}

impl<S: Copy, R: FunctionalRecognizer<S>> StackRecognizer<S, R> {
    pub fn from(rec: R) -> Self {
        let stack = vec![rec.initial(); 300];
        StackRecognizer {
            rec,
            stack,
            stack_ptr: 0,
        }
    }

    pub fn reset(&mut self) {
        self.stack_ptr = 0;
        self.stack[0] = self.rec.initial();
    }

    pub fn recognizer(&self) -> &R {
        &self.rec
    }

    pub fn recognizer_mut(&mut self) -> &mut R {
        &mut self.rec
    }
}

impl<S: Copy + Debug, R: FunctionalRecognizer<S>> Debug for StackRecognizer<S, R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StackRecognizer")
            .field("top", &self.stack[self.stack_ptr])
            .finish()
    }
}

impl<S: Copy + Debug, R: FunctionalRecognizer<S>> Recognizer for StackRecognizer<S, R> {
    #[inline(always)]
    fn pop_bytes(&mut self, num: usize) {
        self.stack_ptr -= num;
    }

    fn trie_finished(&mut self) {
        // println!("{:?}", &self.stack[0..=self.stack_ptr]);
        assert!(self.stack_ptr == 0);
    }

    fn collapse(&mut self) {
        self.stack[0] = self.stack[self.stack_ptr];
        self.stack_ptr = 0;
    }

    fn special_allowed(&mut self, tok: SpecialToken) -> bool {
        self.rec.special_allowed(self.stack[self.stack_ptr], tok)
    }

    #[inline(always)]
    fn try_push_byte(&mut self, byte: u8) -> bool {
        match self.rec.try_append(self.stack[self.stack_ptr], byte) {
            Some(state) => {
                self.stack_ptr += 1;
                self.stack[self.stack_ptr] = state;
                true
            }
            None => false,
        }
    }
}

#[derive(Clone)]
pub struct AnythingGoes {}

impl FunctionalRecognizer<()> for AnythingGoes {
    fn initial(&self) -> () {
        ()
    }

    fn try_append(&self, state: (), _byte: u8) -> Option<()> {
        Some(state)
    }

    fn special_allowed(&self, _state: (), _tok: SpecialToken) -> bool {
        true
    }
}
