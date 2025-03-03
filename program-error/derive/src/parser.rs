//! Token parsing

use {
    proc_macro2::{Ident, Span, TokenStream},
    quote::quote,
    syn::{
        parse::{Parse, ParseStream},
        token::Comma,
        LitInt, LitStr, Token,
    },
};

/// Possible arguments to the `#[spl_program_error]` attribute
pub struct SplProgramErrorArgs {
    /// Whether to hash the error codes using sha-256
    /// or to use the default error code assigned by `num_traits`.
    pub hash_error_code_start: Option<u32>,
    /// Crate to use for solana_program_error
    pub program_error_import: SolanaProgramError,
    /// Crate to use for solana_decode_error
    pub decode_error_import: SolanaDecodeError,
}

/// Struct representing the path to a `solana_program_error` crate, which may
/// be renamed or otherwise.
pub struct SolanaProgramError {
    import: Ident,
    explicit: bool,
}
impl quote::ToTokens for SolanaProgramError {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.import.to_tokens(tokens);
    }
}
impl SolanaProgramError {
    pub fn wrap(&self, output: TokenStream) -> TokenStream {
        if self.explicit {
            output
        } else {
            program_error_anon_const_trick(output)
        }
    }
}
impl Default for SolanaProgramError {
    fn default() -> Self {
        Self {
            import: Ident::new("_solana_program_error", Span::call_site()),
            explicit: false,
        }
    }
}

/// Struct representing the path to a `solana_decode_error` crate, which may
/// be renamed or otherwise.
pub struct SolanaDecodeError {
    import: Ident,
    explicit: bool,
}
impl quote::ToTokens for SolanaDecodeError {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.import.to_tokens(tokens);
    }
}
impl SolanaDecodeError {
    pub fn wrap(&self, output: TokenStream) -> TokenStream {
        if self.explicit {
            output
        } else {
            decode_error_anon_const_trick(output)
        }
    }
}
impl Default for SolanaDecodeError {
    fn default() -> Self {
        Self {
            import: Ident::new("_solana_decode_error", Span::call_site()),
            explicit: false,
        }
    }
}

impl Parse for SplProgramErrorArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut hash_error_code_start = None;
        let mut program_error_import = None;
        let mut decode_error_import = None;
        while !input.is_empty() {
            match SplProgramErrorArgParser::parse(input)? {
                SplProgramErrorArgParser::HashErrorCodes { value, .. } => {
                    hash_error_code_start = Some(value.base10_parse::<u32>()?);
                }
                SplProgramErrorArgParser::SolanaProgramErrorCrate { value, .. } => {
                    program_error_import = Some(SolanaProgramError {
                        import: value.parse()?,
                        explicit: true,
                    });
                }
                SplProgramErrorArgParser::SolanaDecodeErrorCrate { value, .. } => {
                    decode_error_import = Some(SolanaDecodeError {
                        import: value.parse()?,
                        explicit: true,
                    });
                }
            }
        }
        Ok(Self {
            hash_error_code_start,
            program_error_import: program_error_import.unwrap_or(SolanaProgramError::default()),
            decode_error_import: decode_error_import.unwrap_or(SolanaDecodeError::default()),
        })
    }
}

/// Parser for args to the `#[spl_program_error]` attribute
/// ie. `#[spl_program_error(hash_error_code_start = 1275525928)]`
enum SplProgramErrorArgParser {
    HashErrorCodes {
        _equals_sign: Token![=],
        value: LitInt,
        _comma: Option<Comma>,
    },
    SolanaProgramErrorCrate {
        _equals_sign: Token![=],
        value: LitStr,
        _comma: Option<Comma>,
    },
    SolanaDecodeErrorCrate {
        _equals_sign: Token![=],
        value: LitStr,
        _comma: Option<Comma>,
    },
}

impl Parse for SplProgramErrorArgParser {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<Ident>()?;
        match ident.to_string().as_str() {
            "hash_error_code_start" => {
                let _equals_sign = input.parse::<Token![=]>()?;
                let value = input.parse::<LitInt>()?;
                let _comma: Option<Comma> = input.parse().unwrap_or(None);
                Ok(Self::HashErrorCodes {
                    _equals_sign,
                    value,
                    _comma,
                })
            }
            "solana_program_error" => {
                let _equals_sign = input.parse::<Token![=]>()?;
                let value = input.parse::<LitStr>()?;
                let _comma: Option<Comma> = input.parse().unwrap_or(None);
                Ok(Self::SolanaProgramErrorCrate {
                    _equals_sign,
                    value,
                    _comma,
                })
            }
            "solana_decode_error" => {
                let _equals_sign = input.parse::<Token![=]>()?;
                let value = input.parse::<LitStr>()?;
                let _comma: Option<Comma> = input.parse().unwrap_or(None);
                Ok(Self::SolanaDecodeErrorCrate {
                    _equals_sign,
                    value,
                    _comma,
                })
            }
            _ => Err(input.error("Expected argument 'hash_error_code_start', 'solana_program_error', or 'solana_decode_error'")),
        }
    }
}

// Within `exp`, you can bring things into scope with `extern crate`.
//
// We don't want to assume that `solana_program_error::` is in scope - the user
// may have imported it under a different name, or may have imported it in a
// non-toplevel module (common when putting impls behind a feature gate).
//
// Solution: let's just generate `extern crate solana_program_error as
// _solana_program_error` and then refer to `_solana_program_error` in the
// derived code. However, macros are not allowed to produce `extern crate`
// statements at the toplevel.
//
// Solution: let's generate `mod _impl_foo` and import solana_program_error
// within that. However, now we lose access to private members of the
// surrounding module. This is a problem if, for example, we're deriving for a
// newtype, where the inner type is defined in the same module,
// but not exported.
//
// Solution: use the anonymous const trick. For some reason, `extern crate`
// statements are allowed here, but everything from the surrounding module is in
// scope. This trick is taken from serde and num_traits.
fn program_error_anon_const_trick(exp: TokenStream) -> TokenStream {
    quote! {
        const _: () = {
            extern crate solana_program_error as _solana_program_error;
            #exp
        };
    }
}

// Same thing, but for solana_decode_error
fn decode_error_anon_const_trick(exp: TokenStream) -> TokenStream {
    quote! {
        const _: () = {
            extern crate solana_decode_error as _solana_decode_error;
            #exp
        };
    }
}
