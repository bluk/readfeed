//! Provides types to represent elements in an [XML][xml] document.
//!
//! [xml]: https://www.w3.org/TR/2006/REC-xml11-20060816/
use maybe_xml::{token::prop::TagName, Reader};

use crate::Ty;

fn map_tag_name_to_ty(tag_name: TagName<'_>) -> Ty {
    let local_name = tag_name.local().as_str();
    if local_name.trim().eq_ignore_ascii_case("rss") {
        Ty::Rss
    } else if local_name.trim().eq_ignore_ascii_case("feed") {
        Ty::Atom
    } else {
        Ty::XmlOrHtml
    }
}

pub(super) fn find_ty(input: &str) -> Ty {
    Reader::from_str(input)
        .into_iter()
        .find_map(|token| match token.ty() {
            maybe_xml::token::Ty::StartTag(start_tag) => Some(map_tag_name_to_ty(start_tag.name())),
            maybe_xml::token::Ty::EmptyElementTag(empty_tag) => {
                Some(map_tag_name_to_ty(empty_tag.name()))
            }
            maybe_xml::token::Ty::EndTag(_) => Some(Ty::XmlOrHtml),
            maybe_xml::token::Ty::Characters(chars) => {
                if chars.as_str().chars().all(|c| c.is_ascii_whitespace()) {
                    return None;
                }

                Some(Ty::XmlOrHtml)
            }
            maybe_xml::token::Ty::Cdata(cdata) => {
                if cdata
                    .content()
                    .as_str()
                    .chars()
                    .all(|c| c.is_ascii_whitespace())
                {
                    return None;
                }

                Some(Ty::XmlOrHtml)
            }
            maybe_xml::token::Ty::ProcessingInstruction(_)
            | maybe_xml::token::Ty::Declaration(_)
            | maybe_xml::token::Ty::Comment(_) => None,
        })
        .unwrap_or(Ty::Unknown)
}

#[must_use]
pub(crate) fn read_until_end_tag<'a>(
    _namespace: Option<&'a str>,
    local_name: &'a str,
    reader: &Reader<'a>,
    pos: &mut usize,
) -> usize {
    let mut end = *pos;

    while let Some(token) = reader.tokenize(pos) {
        match token.ty() {
            token::Ty::EndTag(tag) => {
                if tag.name().as_str().trim().eq_ignore_ascii_case(local_name) {
                    break;
                }
            }
            token::Ty::StartTag(_)
            | token::Ty::EmptyElementTag(_)
            | token::Ty::Characters(_)
            | token::Ty::ProcessingInstruction(_)
            | token::Ty::Declaration(_)
            | token::Ty::Comment(_)
            | token::Ty::Cdata(_) => {}
        }

        end = *pos;
    }

    end
}

#[must_use]
pub(crate) fn collect_bytes_until_end_tag<'a>(
    namespace: Option<&'a str>,
    local_name: &'a str,
    reader: &Reader<'a>,
    pos: &mut usize,
) -> &'a str {
    let begin = *pos;
    let input = reader.into_inner();
    let end = read_until_end_tag(namespace, local_name, reader, pos);
    &input[begin..end]
}

pub use maybe_xml::token;