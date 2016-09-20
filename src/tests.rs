
macro_rules! assert_match {
    ($left:expr, $right:pat) => {
        match $left {
            $right => true,
            _ => false
        }
    };
}

use rfc5322::{Parsable, ParseError, Streamable};

#[test]
fn test_alpha() {
    use rfc5322::types::Alpha;

    let (alpha, rem) = Alpha::parse(b"abcdEFZz123").unwrap();
    assert_eq!(alpha, Alpha(b"abcdEFZz".to_vec()));
    assert_eq!(rem, b"123");

    let err = Alpha::parse(b"").err().unwrap();
    assert_match!(err, ParseError::Eof);

    let err = Alpha::parse(b"123").err().unwrap();
    assert_match!(err, ParseError::NotFound);

    let mut output: Vec<u8> = Vec::new();
    assert_eq!(alpha.stream(&mut output).unwrap(), 8);
    assert_eq!(output, b"abcdEFZz".to_vec());
}

#[test]
fn test_parse_quoted_pair() {
    use rfc5322::types::QuotedPair;

    let err = QuotedPair::parse(b"not").err().unwrap();
    assert_match!(err, ParseError::NotFound);
    let err = QuotedPair::parse(b"\\").err().unwrap();
    assert_match!(err, ParseError::NotFound);
    let (token, rem) = QuotedPair::parse(b"\\n").unwrap();
    assert_eq!(token, QuotedPair(b'n'));
    assert_eq!(rem, b"");
    let qp = QuotedPair(b'n');
    let mut output: Vec<u8> = Vec::new();
    assert_eq!(qp.stream(&mut output).unwrap(), 2);
    assert_eq!(output, b"\\n");
}

#[test]
fn test_fws() {
    use rfc5322::types::FWS;

    let (token, rem) = FWS::parse(b"   ").unwrap();
    assert_eq!(token, FWS);
    assert_eq!(rem, b"");
    let (token, rem) = FWS::parse(b" \r\n  \t").unwrap();
    assert_eq!(token, FWS);
    assert_eq!(rem, b"");
    let (token, rem) = FWS::parse(b" \r ").unwrap();
    assert_eq!(token, FWS);
    assert_eq!(rem, b"\r ");
    let err = FWS::parse(b"\n ").err().unwrap();
    assert_match!(err, ParseError::NotFound);
    let err = FWS::parse(b"\r\n").err().unwrap();
    assert_match!(err, ParseError::NotFound);
    let (token, rem) = FWS::parse(b"\r\n\tx").unwrap();
    assert_eq!(token, FWS);
    assert_eq!(rem, b"x");
}

#[test]
fn test_ctext() {
    use rfc5322::types::CText;

    let input = b"Thi,s;1:23isCt_#ext".to_vec();
    let (token, remainder) = CText::parse(input.as_slice()).unwrap();
    assert_eq!(token, CText(input.clone()));
    assert_eq!(remainder, b"");
}

#[test]
fn test_ccontent() {
    use rfc5322::types::{CContent, CText, QuotedPair};

    let input = b"Thi,s;1:23isCt_#ext".to_vec();
    let (token, _) = CContent::parse(input.as_slice()).unwrap();
    assert_eq!(token, CContent::CText(CText(input.clone())));

    let input = b"\\n".to_vec();
    let (token, _) = CContent::parse(input.as_slice()).unwrap();
    assert_eq!(token, CContent::QuotedPair(QuotedPair(b'n')));

    let input = b"(Comments can contain whitespace and \\( quoted \\\\ characters, and even ( nesting ) with or (without) whitepsace, but must balance parenthesis)".to_vec();
    let (_, remainder) = CContent::parse(input.as_slice()).unwrap();
    assert_eq!(remainder, b"");
}

#[test]
fn test_comment() {
    use rfc5322::types::{Comment, CContent, CText, QuotedPair};

    let input = b"( a,b,c\t \\nYes (and so on) \r\n )".to_vec();
    let (token, rem) = Comment::parse(input.as_slice()).unwrap();
    assert_eq!(token, Comment {
        ccontent: vec![
            (true, CContent::CText( CText(b"a,b,c".to_vec()) )),
            (true, CContent::QuotedPair( QuotedPair(b'n') )),
            (false, CContent::CText( CText(b"Yes".to_vec()) )),
            (true, CContent::Comment(Comment {
                ccontent: vec![
                    (false, CContent::CText( CText(b"and".to_vec()) )),
                    (true, CContent::CText( CText(b"so".to_vec()) )),
                    (true, CContent::CText( CText(b"on".to_vec()) )) ],
                trailing_ws: false
            }))],
        trailing_ws: true,
    });
    assert_eq!(rem, b"");

    let mut output: Vec<u8> = Vec::new();
    assert_eq!(token.stream(&mut output).unwrap(), 27);
    assert_eq!(output, b"( a,b,c \\nYes (and so on) )");
}

#[test]
fn test_cfws() {
    use rfc5322::types::{CFWS, Comment, CContent, CText, QuotedPair};

    let input = b"  \t( a,b,c\t \\nYes (and so on) \r\n ) \r\n ".to_vec();
    let (token, rem) = CFWS::parse(input.as_slice()).unwrap();
    assert_eq!(token, CFWS {
        comments: vec![
            (true, Comment {
                ccontent: vec![
                    (true, CContent::CText( CText(b"a,b,c".to_vec()) )),
                    (true, CContent::QuotedPair( QuotedPair(b'n') )),
                    (false, CContent::CText( CText(b"Yes".to_vec()) )),
                    (true, CContent::Comment(Comment {
                        ccontent: vec![
                            (false, CContent::CText( CText(b"and".to_vec()) )),
                            (true, CContent::CText( CText(b"so".to_vec()) )),
                            (true, CContent::CText( CText(b"on".to_vec()) )) ],
                        trailing_ws: false
                    }))],
                trailing_ws: true,
            })],
        trailing_ws: true,
    });
    assert_eq!(rem, b"");

    let mut output: Vec<u8> = Vec::new();
    assert_eq!(token.stream(&mut output).unwrap(), 29);
    assert_eq!(output, b" ( a,b,c \\nYes (and so on) ) ");

    let input = b"(abc)(def\r\n )".to_vec();
    let (token, _) = CFWS::parse(input.as_slice()).unwrap();
    assert_eq!(token, CFWS {
        comments: vec![
            (false, Comment {
                ccontent: vec![
                    (false, CContent::CText( CText(b"abc".to_vec()) )) ],
                trailing_ws: false,
            }),
            (false, Comment {
                ccontent: vec![
                    (false, CContent::CText( CText(b"def".to_vec()) )) ],
                trailing_ws: true,
            }),
            ],
        trailing_ws: false,
    });
}

#[test]
fn test_atom() {
    use rfc5322::types::Atom;

    let input = b"  \t( a,b,c\t \\nYes (and so on) \r\n ) atom\r\n ".to_vec();
    let (atom, remainder) = Atom::parse(input.as_slice()).unwrap();
    assert_eq!(atom.atext.0, b"atom".to_vec());
    assert_eq!(remainder, b"");

    let input = b" \t AMZamz019!#$%&'*+-/=?^_`{|}~ \t ".to_vec();
    let (atom, remainder) = Atom::parse(input.as_slice()).unwrap();
    assert_eq!(atom.atext.0, b"AMZamz019!#$%&'*+-/=?^_`{|}~".to_vec());
    assert_eq!(remainder, b"");

    let input = b" John Smith ".to_vec();
    let (atom, remainder) = Atom::parse(input.as_slice()).unwrap();
    assert_eq!(atom.atext.0, b"John".to_vec());
    assert_eq!(remainder, b"Smith ");

    let mut output: Vec<u8> = Vec::new();
    assert_eq!(atom.stream(&mut output).unwrap(), 6);
    assert_eq!(output, b" John ");
}

#[test]
fn test_dot_atom() {
    use rfc5322::types::{DotAtom, AText};

    let input = b" \r\n www.google.com. ".to_vec();
    let (dot_atom, remainder) = DotAtom::parse(input.as_slice()).unwrap();
    assert_eq!(dot_atom.dot_atom_text.0, vec![
        AText(b"www".to_vec()),
        AText(b"google".to_vec()),
        AText(b"com".to_vec())]);
    assert!(dot_atom.pre_cfws.is_some());
    assert!(dot_atom.post_cfws.is_none());
    assert_eq!(remainder, b". ");
}

#[test]
fn test_qcontent() {
    use rfc5322::types::{QContent, QText, QuotedPair};

    let input = b"!#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[]\
                 ^_`abcdefghijklmnopqrstuvwxyz{|}~".to_vec();
    let input2 = input.clone();
    let (token, remainder) = QContent::parse(input.as_slice()).unwrap();
    assert_eq!(token, QContent::QText( QText(input2) ));
    assert_eq!(remainder, b"");

    let input = b"\\nc".to_vec();
    let (token, remainder) = QContent::parse(input.as_slice()).unwrap();
    assert_eq!(token, QContent::QuotedPair( QuotedPair(b'n') ));
    assert_eq!(remainder, b"c");

    let mut output: Vec<u8> = Vec::new();
    assert_eq!(token.stream(&mut output).unwrap(), 2);
    assert_eq!(output, b"\\n");
}

#[test]
fn test_quoted_string() {
    use rfc5322::types::{QuotedString, QContent, QText};

    let input = b" \t (a comment) \" \r\n bob joe\" (fred) ".to_vec();
    let (token, remainder) = QuotedString::parse(input.as_slice()).unwrap();
    assert_eq!(remainder, b"");
    assert!(token.pre_cfws.is_some());
    assert_eq!(token.qcontent, vec![
        (true, QContent::QText( QText(b"bob".to_vec()) )),
        (true, QContent::QText( QText(b"joe".to_vec()) )),
        ]);
    assert_eq!(token.trailing_ws, false);
    assert!(token.post_cfws.is_some());

    let unterminated = b" \t (a comment) \" \r\n bob joe (fred) ".to_vec();
    assert!(QuotedString::parse(unterminated.as_slice()).is_err());
}

#[test]
fn test_phrase() {
    use rfc5322::types::Phrase;

    let input = b" John \"the Snake\" Stevens".to_vec();
    let (phrase, remainder) = Phrase::parse(input.as_slice()).unwrap();
    assert_eq!(phrase.0.len(), 3);
    assert_eq!(remainder, b"");

    let input = b" John Smith [Doctor]".to_vec();
    let (phrase, remainder) = Phrase::parse(input.as_slice()).unwrap();
    assert_eq!(phrase.0.len(), 2);
    assert_eq!(remainder, b"[Doctor]");
}

#[test]
fn test_unstructured() {
    use rfc5322::types::{Unstructured, VChar};

    let input = b"This is; unstructured=5 \r\n ".to_vec();
    let (u, remainder) = Unstructured::parse(input.as_slice()).unwrap();
    assert_eq!(u, Unstructured {
        leading_ws: false,
        parts: vec![
            VChar(b"This".to_vec()),
            VChar(b"is;".to_vec()),
            VChar(b"unstructured=5".to_vec())],
        trailing_ws: true,
    });
    assert_eq!(remainder, b"\r\n "); // because trailing ws is only WSP not FWS
}

#[test]
fn test_domain_literal() {
    use rfc5322::types::{DomainLiteral, DText};

    let input = b"\r\n \t[ 2001:db8:85a3:8d3:1319:8a2e:370:7348]".to_vec();
    let (token, _) = DomainLiteral::parse(input.as_slice()).unwrap();
    assert!(token.pre_cfws.is_some());
    assert_eq!(token.dtext, vec![
        (true, DText(b"2001:db8:85a3:8d3:1319:8a2e:370:7348".to_vec()))
        ]);
    assert_eq!(token.trailing_ws, false);
    assert!(token.post_cfws.is_none());
}

#[test]
fn test_addr_spec() {
    use rfc5322::types::{AddrSpec, LocalPart, Domain, DotAtom, DotAtomText,
                         QuotedString, QContent, DomainLiteral, AText, DText, QText};

    let input = b"joe.smith@gmail.com".to_vec();
    let (a, rem) = AddrSpec::parse(input.as_slice()).unwrap();
    assert_eq!(a.local_part, LocalPart::DotAtom( DotAtom {
        pre_cfws: None,
        dot_atom_text: DotAtomText(vec![ AText(b"joe".to_vec()),
                                         AText(b"smith".to_vec()) ]),
        post_cfws: None,
    }));
    assert_eq!(a.domain, Domain::DotAtom( DotAtom {
        pre_cfws: None,
        dot_atom_text: DotAtomText(vec![ AText(b"gmail".to_vec()),
                                         AText(b"com".to_vec()) ]),
        post_cfws: None,
    }));
    assert_eq!(rem, b"");

    let mut output: Vec<u8> = Vec::new();
    assert_eq!(a.stream(&mut output).unwrap(), 19);
    assert_eq!(output, b"joe.smith@gmail.com".to_vec());

    let input = b"\"joe smith\"@[2001:db8:85a3:8d3:1319:8a2e:370:7348]".to_vec();
    let (a, rem) = AddrSpec::parse(input.as_slice()).unwrap();
    assert_eq!(a.local_part, LocalPart::QuotedString( QuotedString {
        pre_cfws: None,
        qcontent: vec![ (false, QContent::QText(QText(b"joe".to_vec()))),
                         (true, QContent::QText(QText(b"smith".to_vec()))) ],
        trailing_ws: false,
        post_cfws: None,
    }));
    assert_eq!(a.domain, Domain::DomainLiteral( DomainLiteral {
        pre_cfws: None,
        dtext: vec![(false, DText(b"2001:db8:85a3:8d3:1319:8a2e:370:7348".to_vec()))],
        trailing_ws: false,
        post_cfws: None,
    }));
    assert_eq!(rem, b"");
}

#[test]
fn test_angle_addr() {
    use rfc5322::types::AngleAddr;

    let input = b"< admin@example.com >".to_vec();
    let (token, rem) = AngleAddr::parse(input.as_slice()).unwrap();
    assert_eq!(rem, b"");

    let mut output: Vec<u8> = Vec::new();
    assert_eq!(token.stream(&mut output).unwrap(), 21);
    assert_eq!(output, input);
}

#[test]
fn test_name_addr() {
    use rfc5322::types::NameAddr;

    let input = b" Bruce \"The Boss\" < bruce@net> \r\n ".to_vec();
    let (token, rem) = NameAddr::parse(input.as_slice()).unwrap();
    assert_eq!(rem, b"");

    let mut output: Vec<u8> = Vec::new();
    assert_eq!(token.stream(&mut output).unwrap(), 31);
    assert_eq!(output, b" Bruce \"The Boss\" < bruce@net> ".to_vec());
}

#[test]
fn test_mailbox_list() {
    use rfc5322::types::{MailboxList, Mailbox};

    let input = b"a@b.c, \"j p\" <d.e@e.f>,,".to_vec();
    let (mbl, rem) = MailboxList::parse(input.as_slice()).unwrap();
    assert_eq!(mbl.0.len(), 2);
    let mb2 = &mbl.0[1];
    assert_eq!(match mb2 {
        &Mailbox::NameAddr(_) => true,
        &Mailbox::AddrSpec(_) => false,
    }, true);
    assert_eq!(rem, b",,");

    let mut output: Vec<u8> = Vec::new();
    assert_eq!(mbl.stream(&mut output).unwrap(), 22);
    assert_eq!(output, b"a@b.c, \"j p\" <d.e@e.f>".to_vec());
}
