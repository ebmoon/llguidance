from typing import List, Tuple, Mapping, Optional, Sequence, Union, TypedDict, Dict, Any
from ._util import TokenId, StopReason
from ._tokenizer import TokenizerWrapper


class LLTokenizer:
    vocab_size: int
    eos_token: TokenId

    def __new__(
        cls,
        tokenizer: Union[str, TokenizerWrapper],
        n_vocab: Optional[int] = None,
        eos_token: Optional[TokenId] = None,
        slices: Optional[List[str]] = None,
    ) -> "LLTokenizer":
        """
        Create a new tokenizer.
        This contains both the actual tokenizer and the "slices" used as optimization
        when computing the token mask.

        Args:
            tokenizer: str or TokenizerWrapper - if str, it is the name or path to the HF tokenizers tokenizer; otherwise it is a TokenizerWrapper
            n_vocab: int - override the size of the vocabulary
            slices: List[str] - configuration for slicer optimization; pass [] to disable,
                or None to use general_slices()
        """

    def with_slices(self, slices: List[str]) -> "LLTokenizer":
        """
        Create a new tokenizer with the specified "slices" for optimization when computing the token mask.
        """

    @staticmethod
    def general_slices() -> List[str]:
        """
        Get the default slices for optimization when computing the token mask.
        This should be good for most grammars.
        """

    @staticmethod
    def json_slices() -> List[str]:
        """
        Get the slices suitable for JSON Schema grammars.
        """

    def greedy_tokenize(self, text: str) -> List[int]:
        """
        Tokenize the text using a greedy algorithm.
        This will not necessarily match BPE.
        """

    def tokenize_bytes(self, utf8bytes: bytes) -> List[int]:
        """
        Tokenize the text as bytes.
        This will use the underlying Python tokenizer to tokenize valid UTF8
        prefix of the text, and then fallback to greedy_tokenize() for the last
        few bytes.
        """

    def tokenize_str(self, text: str) -> List[int]:
        """
        Same as tokenize_bytes, but for strings.
        """

    def dbg_tokens(self, tokens: List[int]) -> str:
        """
        Return a debug string representation of the tokens.
        The result is double-quoted and tokens are separated by '‧'.
        """

    def test_trace_tokens(self, tokens: List[int]) -> str:
        """
        Return a debug string representation of the tokens
        for test traces.
        """

    def decode_str(self, tokens: List[int]) -> str:
        """
        Decode the tokens into a string.
        Invalid UTF-8 will be replaced with the Unicode replacement character.
        """

    def decode_bytes(self, tokens: List[int]) -> bytes:
        """
        Decode the tokens into a bytes object.
        """

    def is_special_token(self, token: int) -> bool:
        """
        Check if the token is a special token.
        """


class LLInterpreter:

    def __new__(
        cls,
        tokenizer: LLTokenizer,
        grammar: str,
        enable_backtrack: bool = True,
        enable_ff_tokens: bool = True,
        log_level: int = 1,
    ) -> "LLInterpreter":
        """
        Create a new interpreter.
        Args:
            tokenizer: LLTokenizer - the tokenizer to use
            grammar: str - either a Lark grammar or stringified JSON representation of LLGuidance grammar
            enable_backtrack: bool - whether to enable backtracking in the interpreter
            enable_ff_tokens: bool - whether to enable fast-forwarded tokens in the interpreter
            log_level: int - the verbosity level of the interpreter
                0 is silent, 1 is warnings, 2 is verbose
        """

    def deep_copy(self) -> "LLInterpreter":
        """
        Create a deep copy of the interpreter.
        """

    def is_accepting(self) -> bool:
        """
        Check if the last compute_mask() call resulted in overall accepting state
        of the parser.
        """

    def stop_reason(self) -> StopReason:
        """
        Get the reason why the parser stopped.
        Returns:
            "NotStopped" - Parser has not emitted stop() yet.
            "MaxTokensTotal" - max_tokens limit on the total number of tokens has been reached.
            "MaxTokensParser" - max_tokens limit on the number of tokens in the top-level parser has been reached.
            "ParserTooComplex" - Grammar is too complex (row item limit)
            "LexerTooComplex" - Lexer regex hit some limit
            "NoExtension" - Top-level parser indicates that no more bytes can be added.
            "NoExtensionBias" - Top-level parser indicates that no more bytes can be added, however it was recognized late.
            "EndOfSentence" - Top-level parser allowed EOS (as it was in an accepting state), and EOS was generated.
            "InternalError" - Something went wrong with creating a nested parser.
        """

    def process_prompt(self, prompt: List[TokenId]) -> List[TokenId]:
        """
        Perform any adjustments to the prompt before completion.
        Returns the adjusted prompt.
        """

    def start_without_prompt(self) -> None:
        """
        Start the parser without prompt processing.
        """

    def validate_tokens_raw(self, tokens: List[TokenId]) -> int:
        """
        Check if tokens are valid in the current state.
        Note that this doesn't currently check for max_tokens beyond the first token (hence 'raw').
        Return: how many of the tokens in the list can be committed
        """

    def compute_mask(self) -> Tuple[Optional[bytes], str]:
        """
        Perform next parsing step.
        Returns: optional token mask and a JSON string.
        """

    def compute_mask_into(self, trg: bytearray) -> str:
        """
        Perform next parsing step.
        Returns: a JSON string.
        """

    def unsafe_compute_mask_ptr(self, trg_pointer: int,
                                trg_byte_size: int) -> str:
        """
        Perform next parsing step.
        Returns: a JSON string.
        """

    def commit_token(
            self,
            sampled_token: Optional[TokenId]) -> Tuple[int, List[TokenId]]:
        """
        Perform any adjustments to the sampled token.
        Returns the number of tokens to remove from the prompt and the
        list of tokens to append.
        If compute_mask() returned None mask, this should be called immediately with None.
        If compute_mask() returned stop, you don't need to call this (but can).
        """

    def has_pending_stop(self) -> bool:
        """
        If true, next compute_mask() call will return stop
        """


class LLMatcher:

    def __new__(cls,
                tokenizer: LLTokenizer,
                grammar: str,
                log_level: int = 1) -> "LLMatcher":
        """
        Create a new LLMatcher.
        Args:
            tokenizer: LLTokenizer - the tokenizer to use
            grammar: str - either a Lark grammar or stringified JSON representation of LLGuidance grammar
            log_level: int - verbosity level (0: silent, 1: warnings, 2: verbose)
        Raises:
            ValueError: if the grammar is invalid.

        Note:
            Other methods in this class do not raise exceptions for user (grammar) errors,
            resource limits, or when an invalid token is consumed.
            In such cases, the matcher will enter an error state, and never leave it.
            You can use is_error() and get_error() to check for the error.
            Methods will raise exceptions when misused at the API level.

        Note:
            This drops the GIL for the duration of the grammar construction, which can be
            100-1000ms for extremely complex grammars.
        """

    @staticmethod
    def grammar_from_json_schema(
        schema: Union[str, Dict[str, Any]],
        /,
        defaults: Optional[JsonCompileOptions] = None,
        overrides: Optional[JsonCompileOptions] = None,
    ) -> str:
        """
        Create a grammar from a JSON schema.

        Args:
            schema: str or dict - the JSON schema; can be stringified already or not
            defaults, overrides: JsonCompileOptions - options for the JSON compiler;
                they are applied in order: defaults -> schema["x-guidance"] -> overrides
        
        Raises:
            ValueError: if either of the arguments is not a valid JSON object.
            This does not check for schema validity.
            LLMatcher constructor will raise if the grammar is invalid.
        """

    @staticmethod
    def grammar_from_lark(lark: str) -> str:
        """
        Create a grammar from a Lark grammar.
        This never raises exceptions (it doesn't check for grammar validity).
        LLMatcher constructor will raise if the grammar is invalid.
        """

    @staticmethod
    def grammar_from_regex(regex: str) -> str:
        """
        Create a grammar from a regex.
        This never raises exceptions (it doesn't check for regex validity).
        LLMatcher constructor will raise if the regex is invalid.
        """

    def is_error(self) -> bool:
        """
        Check if the matcher is in an error state.
        Once matcher is in an error state, it will stay in it.
        """

    def get_error(self) -> str:
        """
        Get the error message if the matcher is in an error state, empty string otherwise.
        """

    def deep_copy(self) -> "LLMatcher":
        """
        Create a deep copy of the matcher.
        """

    def is_accepting(self) -> bool:
        """
        Check if the matcher is in an accepting state (can be terminated and the grammar is satisfied).
        """

    def is_stopped(self) -> bool:
        """
        Check if the matcher is stopped, and will not accept any more tokens, except for the EOS token.
        This is also true when matcher is in an error state, use is_error() or get_error() to check for that.
        """

    def stop_reason(self) -> StopReason:
        """
        Get the reason why the matcher stopped.
        May return "NotStopped" if the matcher is not stopped.
        """

    def rollback(self, num_tokens: int) -> None:
        """
        Rollback the last num_tokens consumed.
        """

    def compute_ff_tokens(self) -> List[TokenId]:
        """
        Compute and return the fast-forward tokens available in the current state.
        """

    def compute_ff_bytes(self) -> bytes:
        """
        Compute and return the forced bytes available in the current state.
        """

    def try_consume_tokens(self, tokens: List[TokenId]) -> int:
        """
        Try consuming a list of tokens and return how many were successfully consumed.
        """

    def consume_token(self, sampled_token: TokenId) -> bool:
        """
        Consume a single token.
        Returns true on success.
        If it returns false, the matcher is in an error state (either from previous errors or it has just entered it).
        """

    def consume_tokens(self, sampled_tokens: List[TokenId]) -> bool:
        """
        Consume a list of tokens.
        Returns true on success.
        If it returns false, the matcher is in an error state (either from previous errors or it has just entered it).
        """

    def validate_tokens(self, tokens: List[TokenId]) -> int:
        """
        Check how many of the tokens in the list can be committed in the current state.
        """

    def compute_bitmask(self) -> bytes:
        """
        Compute the token mask, with one bit per tokenizer word, for the next parsing step.
        This drops the GIL, but for best performance, use fill_next_token_bitmask_par().
        """

    def compute_logit_bias(self) -> bytes:
        """
        Compute the token mask, with one byte per tokenizer word, for the next parsing step.
        Entries are either 0 (not allowed) or 200 (allowed).
        The idea is to create a uint8 tensor from the result, convert to float and add to logits tensor.
        This drops the GIL, but for best performance, use fill_next_token_bitmask_par()
        and apply_token_bitmask().
        """

    def unsafe_compute_mask_ptr(self, trg_pointer: int,
                                trg_byte_size: int) -> None:
        """
        Compute the token mask directly into memory at the specified pointer.
        This drops the GIL; prefer to use fill_next_token_bitmask() or fill_next_token_bitmask_par().
        """


class JsonCompiler:

    def __new__(cls,
                separators: Optional[Tuple[str, str]] = None,
                whitespace_flexible: bool = False,
                coerce_one_of: bool = False,
                whitespace_pattern: Optional[str] = None) -> "JsonCompiler":
        """
        Create a new JSON compiler.
        """

    def compile(
        self,
        schema: str,
        check: bool = True,
    ) -> str:
        """
        Similar to:

            g = LLMatcher.grammar_from_json_schema(schema, overrides=self.options)
            if check:
                LLMatcher(tokenizer, g)

        Best not use.
        """


class LarkCompiler:

    def __new__(cls, ) -> "LarkCompiler":
        """
        Create a new Lark compiler.
        """

    def compile(
        self,
        lark: str,
        check: bool = True,
    ) -> str:
        """
        Equivalent to (with an empty tokenizer):

            g = LLMatcher.grammar_from_lark(lark)
            if check:
                LLMatcher(tokenizer, g)

        Best not use.
        """


class RegexCompiler:

    def __new__(cls) -> "RegexCompiler":
        """
        Create a new Regex compiler.
        """

    def compile(
        self,
        regex: str,
        check: bool = True,
    ) -> str:
        """
        Equivalent to:

            g = LLMatcher.grammar_from_regex(regex)
            if check:
                LLMatcher(tokenizer, g)

        Best not use.
        """


class LLExecutor:

    def __new__(
        cls,
        num_threads: Optional[int] = None,
    ) -> "LLExecutor":
        """
        Create a new executor.
        Args:
            num_threads: int - number of threads to use for parallel execution,
                or None to use the default number of threads (80% of the available CPUs up to 32)
        """

    def unsafe_compute_mask_ptr(
        self,
        interpreters: List[Tuple[LLMatcher, int]],
        trg_pointer: int,
        one_mask_byte_size: int,
        trg_batch_size: int,
    ) -> None:
        """
        Compute the token mask directly into memory at the specified pointer.
        For each matcher, provide the index of the target mask.
        If index is K, the memory will be written at trg_pointer + K * one_mask_byte_size,
        where K < trg_batch_size.
        Memory has to have size trg_batch_size * one_mask_byte_size.
        Prefer to use fill_next_token_bitmask_par(), which wraps this.
        """


class JsonCompileOptions(TypedDict, total=False):
    # defaults to ","
    item_separator: Optional[str]
    # defaults to ":"
    key_separator: Optional[str]
    # defaults to None - depends on whitespace_flexible
    whitespace_pattern: Optional[str]
    # defaults to true (r"[\x20\x0A\x0D\x09]+"); if false, no whitespace is allowed
    whitespace_flexible: Optional[bool]
    # defaults to false
    coerce_one_of: Optional[bool]
