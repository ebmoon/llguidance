use aici_abi::{tokenize_bytes, toktree::TokTrie, MidProcessArg, MidProcessResult, TokenId};
use anyhow::Result;
use earley::Parser;

use crate::earley::{earley_grm_from_guidance, ParseResult};

pub mod earley;
mod serialization;

const INFO: bool = true;

macro_rules! infoln {
    ($($arg:tt)*) => {
        if INFO {
            println!($($arg)*);
        }
    };
}

pub struct TokenParser {
    toktrie: TokTrie,
    pub parser: Parser,
    // tokens currently in KV cache
    llm_tokens: Vec<TokenId>,
}

impl TokenParser {
    pub fn from_guidance_protobuf(buf: &[u8]) -> Result<Self> {
        let grm = earley_grm_from_guidance(buf)?;
        infoln!("original: {:?}", grm);
        let grm = grm.optimize();
        infoln!("optimized: {:?}", grm);
        let cgrm = grm.compile();
        let parser = Parser::new(cgrm);
        Ok(TokenParser {
            toktrie: TokTrie::from_host(),
            parser,
            llm_tokens: Vec::new(),
        })
    }

    pub fn mid_process(&mut self, arg: MidProcessArg) -> MidProcessResult {
        infoln!("post tokens: {}", self.toktrie.tokens_dbg(&arg.tokens));
        arg.save_tokens(&mut self.llm_tokens);

        if arg.tokens.contains(&self.toktrie.eos_token()) {
            return MidProcessResult::stop();
        }

        let start_time = std::time::Instant::now();
        let _ = self.parser.force_bytes();
        // tokens/bytes forced by the grammar
        let full_grm_bytes = self.parser.get_bytes();
        let mut grm_tokens = tokenize_bytes(&full_grm_bytes);
        let mut suff = Vec::new();
        let mut chop_tokens = 0;
        let mut chop_bytes = 0;
        for (idx, t) in grm_tokens.iter().rev().enumerate() {
            suff.splice(0..0, self.toktrie.token(*t).iter().cloned());
            if suff.len() > self.toktrie.max_token_len() {
                break;
            }
            // TODO do we have to consider if the extensions match the grammar?
            if self.toktrie.has_extensions(&suff) {
                chop_tokens = idx + 1;
                chop_bytes = suff.len();
            }
        }
        // here we remove a suffix from grm_tokens that could be possibly tokenized differently
        grm_tokens.truncate(grm_tokens.len() - chop_tokens);

        for idx in 0..grm_tokens.len() {
            // if the LLM state disagrees with forced tokens, we need to splice
            if self.llm_tokens.get(idx) != grm_tokens.get(idx) {
                let backtrack: u32 = (self.llm_tokens.len() - idx).try_into().unwrap();
                let ff_tokens = grm_tokens[idx..].to_vec();
                infoln!(
                    "backtrack: {}, ff_tokens: {}",
                    backtrack,
                    self.toktrie.tokens_dbg(&ff_tokens),
                );
                infoln!("fixed_tokens: {:?}", self.toktrie.tokens_dbg(&grm_tokens));
                return MidProcessResult::splice(backtrack, ff_tokens);
            }
        }

        // here, grm_tokens are at most as long as llm_tokens (otherwise we would have spliced)
        // llm_suffix are additional bytes generated by the model
        let llm_suffix = self.toktrie.decode(&self.llm_tokens[grm_tokens.len()..]);
        // grm_suffix are additional bytes generated by the grammar
        let grm_suffix = full_grm_bytes[full_grm_bytes.len() - chop_bytes..].to_vec();

        let byte_suffix = if grm_suffix.len() <= llm_suffix.len() {
            if !llm_suffix.starts_with(&grm_suffix) {
                panic!(
                    "llm_suffix: {:?}, grm_suffix: {:?} (grm<=llm)",
                    String::from_utf8_lossy(&llm_suffix),
                    String::from_utf8_lossy(&grm_suffix)
                );
            }

            for b in &llm_suffix[grm_suffix.len()..] {
                let r = self.parser.scan(*b);
                if r == ParseResult::Reject {
                    panic!("rejected byte: {}", b);
                }
            }
            vec![]
        } else {
            if !grm_suffix.starts_with(&llm_suffix) {
                panic!(
                    "llm_suffix: {:?}, grm_suffix: {:?} (grm>llm)",
                    String::from_utf8_lossy(&llm_suffix),
                    String::from_utf8_lossy(&grm_suffix)
                );
            }
            grm_suffix[llm_suffix.len()..].to_vec()
        };

        // self.parser.print_row(self.parser.num_rows() - 1);

        let mut set = self.toktrie.alloc_token_set();
        self.toktrie
            .compute_bias_ext(&mut self.parser, &mut set, &byte_suffix);
        infoln!(
            "bias: (pref: {:?}) {:?} {}",
            String::from_utf8_lossy(&byte_suffix),
            start_time.elapsed(),
            self.toktrie.token_set_dbg(&set)
        );

        return MidProcessResult::sample(set);
    }
}
