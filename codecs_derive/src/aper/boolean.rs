//! `APER` Code generation for ASN.1 Boolean Type

use proc_macro::TokenStream;
use quote::quote;

use crate::attrs::TyCodecParams;

pub(super) fn generate_aper_codec_for_asn_boolean(
    ast: &syn::DeriveInput,
    _params: &TyCodecParams,
) -> proc_macro::TokenStream {
    let name = &ast.ident;

    let tokens = quote! {

        impl asn1_codecs::aper::AperCodec for #name {
            fn decode(data: &mut asn1_codecs::aper::AperCodecData) -> Result<Self, asn1_codecs::aper::AperCodecError> {

                let value = asn1_codecs::aper::decode::decode_bool(data)?;
                Ok(Self(value))
            }

            fn encode(&self, data: &mut asn1_codecs::aper::AperCodecData) -> Result<(), asn1_codecs::aper::AperCodecError> {
                asn1_codecs::aper::encode::encode_bool(data, self.0)
            }
        }
    };

    TokenStream::from(tokens)
}
