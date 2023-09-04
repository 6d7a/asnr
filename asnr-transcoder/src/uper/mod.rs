use bitvec::{bitvec, prelude::Msb0, vec::BitVec, view::BitView};
use bitvec_nom::BSlice;

use alloc::{string::String, vec::Vec};

use crate::{
    error::{DecodingError, EncodingError},
    Decode, Encode,
};

mod decoder;
mod encoder;

pub struct Uper;

impl Uper {
    pub fn decode<'a, T: Decode<'a, BitIn<'a>>>(
        input: &'a [u8],
    ) -> Result<T, DecodingError<BitIn>> {
        T::decode::<Uper>(BitIn::from(input.view_bits::<Msb0>())).map(|(_, res)| res)
    }

    pub fn encode<'a, T: Encode<u8, BitOut>>(input: T) -> Result<Vec<u8>, EncodingError> {
        T::encode::<Uper>(input, bitvec![u8, Msb0;]).map(|mut bitvec| {
            bitvec.set_uninitialized(false);
            bitvec.into_vec()
        })
    }
}

pub type BitIn<'a> = BSlice<'a, u8, Msb0>;
pub type BitOut = BitVec<u8, Msb0>;

const RUST_KEYWORDS: [&'static str; 38] = [
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub",
    "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true", "type",
    "unsafe", "use", "where", "while",
];

pub fn rustify_name(name: &String) -> String {
    let name = name.replace("-", "_");
    if RUST_KEYWORDS.contains(&name.as_str()) {
        String::from("r_") + &name
    } else {
        name
    }
}

#[cfg(test)]
mod tests {
    use asnr_compiler_derive::asn1_internal_tests;

    use crate::uper::Uper;

    #[test]
    fn encodes_as_decodes_integer() {
        asn1_internal_tests!(
            r#"Int-1 ::= INTEGER
            Int-2 ::= INTEGER(42)
            Int-3 ::= INTEGER(-1..65355)
            Int-4 ::= INTEGER(23..MAX)
            Int-5 ::= INTEGER(20,...)
            Int-6 ::= INTEGER(1..24,...)"#
        );

        assert_eq!(
            42,
            Uper::decode::<Int_1>(&Uper::encode(Int_1(42)).unwrap())
                .unwrap()
                .0
        );
        assert_eq!(
            42,
            Uper::decode::<Int_2>(&Uper::encode(Int_2(42)).unwrap())
                .unwrap()
                .0
        );
        assert_eq!(
            42,
            Uper::decode::<Int_3>(&Uper::encode(Int_3(42)).unwrap())
                .unwrap()
                .0
        );
        assert_eq!(
            42,
            Uper::decode::<Int_4>(&Uper::encode(Int_4(42)).unwrap())
                .unwrap()
                .0
        );
        assert_eq!(
            87000,
            Uper::decode::<Int_5>(&Uper::encode(Int_5(87000)).unwrap())
                .unwrap()
                .0
        );
        assert_eq!(
            42,
            Uper::decode::<Int_6>(&Uper::encode(Int_6(42)).unwrap())
                .unwrap()
                .0
        );
    }

    #[test]
    fn encodes_as_decodes_bit_string() {
        asn1_internal_tests!(
            r#"Bit-string-1 ::= BIT STRING 
            Bit-string-2 ::= BIT STRING (SIZE(4))
            Bit-string-3 ::= BIT STRING (SIZE(1..63))
            Bit-string-4 ::= BIT STRING (SIZE(2,...))
            Bit-string-5 ::= BIT STRING (SIZE(2..24,...))"#
        );

        assert_eq!(
            vec![true],
            Uper::decode::<Bit_string_1>(&Uper::encode(Bit_string_1(vec![true])).unwrap())
                .unwrap()
                .0
        );
        assert_eq!(
            vec![true, false, true, false],
            Uper::decode::<Bit_string_2>(
                &Uper::encode(Bit_string_2(vec![true, false, true, false])).unwrap()
            )
            .unwrap()
            .0
        );
        assert_eq!(
            vec![true, false],
            Uper::decode::<Bit_string_3>(&Uper::encode(Bit_string_3(vec![true, false])).unwrap())
                .unwrap()
                .0
        );
        assert_eq!(
            vec![true],
            Uper::decode::<Bit_string_4>(&Uper::encode(Bit_string_4(vec![true])).unwrap())
                .unwrap()
                .0
        );
        assert_eq!(
            vec![true],
            Uper::decode::<Bit_string_5>(&Uper::encode(Bit_string_5(vec![true])).unwrap())
                .unwrap()
                .0
        );
        assert_eq!(
            vec![true, false],
            Uper::decode::<Bit_string_5>(&Uper::encode(Bit_string_5(vec![true, false])).unwrap())
                .unwrap()
                .0
        );
    }

    #[test]
    fn encodes_as_decodes_octet_string() {
        asn1_internal_tests!(
            r#"Octet-string-1 ::= OCTET STRING 
            Octet-string-2 ::= OCTET STRING (SIZE(4))
            Octet-string-3 ::= OCTET STRING (SIZE(1..63))
            Octet-string-4 ::= OCTET STRING (SIZE(2,...))
            Octet-string-5 ::= OCTET STRING (SIZE(2..24,...))"#
        );

        assert_eq!(
            vec![22],
            Uper::decode::<Octet_string_1>(&Uper::encode(Octet_string_1(vec![22])).unwrap())
                .unwrap()
                .0
        );
        assert_eq!(
            vec![22, 55, 33, 44],
            Uper::decode::<Octet_string_2>(
                &Uper::encode(Octet_string_2(vec![22, 55, 33, 44])).unwrap()
            )
            .unwrap()
            .0
        );
        assert_eq!(
            vec![33, 77],
            Uper::decode::<Octet_string_3>(&Uper::encode(Octet_string_3(vec![33, 77])).unwrap())
                .unwrap()
                .0
        );
        assert_eq!(
            vec![22],
            Uper::decode::<Octet_string_4>(&Uper::encode(Octet_string_4(vec![22])).unwrap())
                .unwrap()
                .0
        );
        assert_eq!(
            vec![22],
            Uper::decode::<Octet_string_5>(&Uper::encode(Octet_string_5(vec![22])).unwrap())
                .unwrap()
                .0
        );
        assert_eq!(
            vec![33, 44],
            Uper::decode::<Octet_string_5>(&Uper::encode(Octet_string_5(vec![33, 44])).unwrap())
                .unwrap()
                .0
        );
    }

    #[test]
    fn encodes_as_decodes_ia5string() {
        asn1_internal_tests!(
            r#"String-1 ::= IA5String
        String-2 ::= IA5String(SIZE(4))
        String-3 ::= IA5String(SIZE(1..63))
        String-4 ::= IA5String(SIZE(20,...))
        String-5 ::= IA5String(SIZE(1..24,...))"#
        );

        assert_eq!(
            "Hello",
            &Uper::decode::<String_1>(&Uper::encode(String_1("Hello".into())).unwrap())
                .unwrap()
                .0
        );
        assert_eq!(
            "WRLD",
            &Uper::decode::<String_2>(&Uper::encode(String_2("WRLD".into())).unwrap())
                .unwrap()
                .0
        );
        assert_eq!(
            "Hello, World!",
            &Uper::decode::<String_3>(&Uper::encode(String_3("Hello, World!".into())).unwrap())
                .unwrap()
                .0
        );
        assert_eq!(
            "Hello, ASN1!",
            &Uper::decode::<String_4>(&Uper::encode(String_4("Hello, ASN1!".into())).unwrap())
                .unwrap()
                .0
        );
        assert_eq!(
            "Hello, Abstract Syntax Notation 1!",
            &Uper::decode::<String_5>(
                &Uper::encode(String_5("Hello, Abstract Syntax Notation 1!".into())).unwrap()
            )
            .unwrap()
            .0
        );
    }

    #[test]
    fn encodes_as_decodes_sequence_of() {
        asn1_internal_tests!(
            r#"
        Sequence-of-1 ::= SEQUENCE OF Member
        Sequence-of-2 ::= SEQUENCE (SIZE(4)) OF Member
        Sequence-of-3 ::= SEQUENCE (SIZE(1..63)) OF Member
        Sequence-of-4 ::= SEQUENCE (SIZE(2,...)) OF Member
        Sequence-of-5 ::= SEQUENCE (SIZE(1..24,...)) OF Member
        Member ::= BOOLEAN"#
        );

        assert_eq!(
            vec![Member(true)],
            Uper::decode::<Sequence_of_1>(
                &Uper::encode(Sequence_of_1(vec![Member(true)])).unwrap()
            )
            .unwrap()
            .0
        );
        assert_eq!(
            vec![Member(true), Member(true), Member(false), Member(true)],
            Uper::decode::<Sequence_of_2>(
                &Uper::encode(Sequence_of_2(vec![
                    Member(true),
                    Member(true),
                    Member(false),
                    Member(true)
                ]))
                .unwrap()
            )
            .unwrap()
            .0
        );
        assert_eq!(
            vec![Member(true)],
            Uper::decode::<Sequence_of_3>(
                &Uper::encode(Sequence_of_3(vec![Member(true)])).unwrap()
            )
            .unwrap()
            .0
        );
        assert_eq!(
            vec![Member(true)],
            Uper::decode::<Sequence_of_4>(
                &Uper::encode(Sequence_of_4(vec![Member(true)])).unwrap()
            )
            .unwrap()
            .0
        );
        assert_eq!(
            Vec::<Member>::new(),
            Uper::decode::<Sequence_of_5>(&Uper::encode(Sequence_of_5(vec![])).unwrap())
                .unwrap()
                .0
        );
    }

    #[test]
    fn encodes_as_decodes_sequence() {
        asn1_internal_tests!(
            r#"
        Seq-1 ::= SEQUENCE {
            member1 Member-1
        }

        Seq-2 ::= SEQUENCE {
            member1 Member-1,
            ...,
            ext1 Member-2
        }

        Seq-3 ::= SEQUENCE {
            member1 Member-1 DEFAULT TRUE,
            member2 Member-2 DEFAULT 0,
            member3 Member-3 OPTIONAL,
            member4 Member-4 OPTIONAL
        }

        Seq-4 ::= SEQUENCE {
            member1 Member-1 DEFAULT TRUE,
            member2 Member-2 DEFAULT 0,
            member3 Member-3 OPTIONAL,
            ...
            ext1 Member-4 OPTIONAL
        }
        
        Member-1 ::= BOOLEAN
        Member-2 ::= INTEGER(0..2)
        Member-3 ::= SEQUENCE OF Member-1
        Member-4 ::= BIT STRING (SIZE(1))"#
        );

        assert_eq!(
            Seq_1 {
                member1: Member_1(true)
            },
            Uper::decode::<Seq_1>(
                &Uper::encode(Seq_1 {
                    member1: Member_1(true)
                })
                .unwrap()
            )
            .unwrap()
        );

        assert_eq!(
            Seq_2 {
                member1: Member_1(true),
                ext1: None,
            },
            Uper::decode::<Seq_2>(
                &Uper::encode(Seq_2 {
                    member1: Member_1(true),
                    ext1: None,
                })
                .unwrap()
            )
            .unwrap()
        );

        assert_eq!(
            Seq_2 {
                member1: Member_1(true),
                ext1: Some(Member_2(1)),
            },
            Uper::decode::<Seq_2>(
                &Uper::encode(Seq_2 {
                    member1: Member_1(true),
                    ext1: Some(Member_2(1)),
                })
                .unwrap()
            )
            .unwrap()
        );

        assert_eq!(
            Seq_3 {
                member1: None,
                member2: Some(Member_2(1)),
                member3: None,
                member4: Some(Member_4(vec![false]))
            },
            Uper::decode::<Seq_3>(
                &Uper::encode(Seq_3 {
                    member1: None,
                    member2: Some(Member_2(1)),
                    member3: None,
                    member4: Some(Member_4(vec![false]))
                })
                .unwrap()
            )
            .unwrap()
        );

        assert_eq!(
            Seq_3 {
                member1: Some(Member_1(false)),
                member2: Some(Member_2(1)),
                member3: Some(Member_3(vec![Member_1(true)])),
                member4: Some(Member_4(vec![false]))
            },
            Uper::decode::<Seq_3>(
                &Uper::encode(Seq_3 {
                    member1: Some(Member_1(false)),
                    member2: Some(Member_2(1)),
                    member3: Some(Member_3(vec![Member_1(true)])),
                    member4: Some(Member_4(vec![false]))
                })
                .unwrap()
            )
            .unwrap()
        );

        assert_eq!(
            Seq_3 {
                member1: None,
                member2: None,
                member3: None,
                member4: None,
            },
            Uper::decode::<Seq_3>(
                &Uper::encode(Seq_3 {
                    member1: None,
                    member2: None,
                    member3: None,
                    member4: None,
                })
                .unwrap()
            )
            .unwrap()
        );

        assert_eq!(
            Seq_4 {
                member1: None,
                member2: Some(Member_2(1)),
                member3: None,
                ext1: Some(Member_4(vec![false]))
            },
            Uper::decode::<Seq_4>(
                &Uper::encode(Seq_4 {
                    member1: None,
                    member2: Some(Member_2(1)),
                    member3: None,
                    ext1: Some(Member_4(vec![false]))
                })
                .unwrap()
            )
            .unwrap()
        );

        assert_eq!(
            Seq_4 {
                member1: Some(Member_1(false)),
                member2: Some(Member_2(1)),
                member3: Some(Member_3(vec![Member_1(true)])),
                ext1: Some(Member_4(vec![false]))
            },
            Uper::decode::<Seq_4>(
                &Uper::encode(Seq_4 {
                    member1: Some(Member_1(false)),
                    member2: Some(Member_2(1)),
                    member3: Some(Member_3(vec![Member_1(true)])),
                    ext1: Some(Member_4(vec![false]))
                })
                .unwrap()
            )
            .unwrap()
        );

        assert_eq!(
            Seq_4 {
                member1: Some(Member_1(false)),
                member2: Some(Member_2(1)),
                member3: Some(Member_3(vec![Member_1(true)])),
                ext1: None
            },
            Uper::decode::<Seq_4>(
                &Uper::encode(Seq_4 {
                    member1: Some(Member_1(false)),
                    member2: Some(Member_2(1)),
                    member3: Some(Member_3(vec![Member_1(true)])),
                    ext1: None
                })
                .unwrap()
            )
            .unwrap()
        );

        assert_eq!(
            Seq_4 {
                member1: None,
                member2: None,
                member3: None,
                ext1: None,
            },
            Uper::decode::<Seq_4>(
                &Uper::encode(Seq_4 {
                    member1: None,
                    member2: None,
                    member3: None,
                    ext1: None,
                })
                .unwrap()
            )
            .unwrap()
        );
    }
}
