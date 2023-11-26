//! OPML is an XML format for outlines. It may be used to export and import a
//! list of feeds.
//!
//! Use [`Iter`] as the starting type for parsing a feed.
//!
//! ## Examples
//!
//! ### OPML
//!
//! ```rust
//! use readfeed::opml::{self, BodyElem, Elem, OpmlElem, OutlineElem};
//!
//! let input = r#"
//! <opml>
//! <body>
//! <outline
//!   text="Example Blog Site"
//!   title="Example Blog"
//!   description="An example."
//!   type="rss"
//!   version="RSS"
//!   htmlUrl="https://blog.example.com/"
//!   xmlUrl="https://blog.example.com/index.xml"
//! />
//! </body>
//! </opml>
//! "#;
//!
//! let mut iter = opml::Iter::new(input);
//!
//! let Some(Elem::Opml(mut opml_iter)) = iter.next() else {
//!     panic!();
//! };
//!
//! let Some(OpmlElem::Body(mut body_iter)) = opml_iter.next() else {
//!     panic!();
//! };
//!
//! let Some(BodyElem::Outline(mut outline_iter)) = body_iter.next() else {
//!     panic!();
//! };
//!
//! assert_eq!(Some("Example Blog Site"), outline_iter.text().map(|v| v.as_str()));
//! assert_eq!(Some("Example Blog"), outline_iter.title().map(|v| v.as_str()));
//! assert_eq!(Some("An example."), outline_iter.description().map(|v| v.as_str()));
//! assert_eq!(Some("rss"), outline_iter.ty().map(|v| v.as_str()));
//! assert_eq!(Some("RSS"), outline_iter.version().map(|v| v.as_str()));
//! assert_eq!(Some("https://blog.example.com/"), outline_iter.html_url().map(|v| v.as_str()));
//! assert_eq!(Some("https://blog.example.com/index.xml"), outline_iter.xml_url().map(|v| v.as_str()));
//!
//! assert_eq!(None, body_iter.next());
//! assert_eq!(None, opml_iter.next());
//! assert_eq!(None, iter.next());
//! ```
//!
//! [opml]: https://en.wikipedia.org/wiki/OPML

use maybe_xml::{
    token::{
        self,
        prop::{AttributeValue, Attributes},
        Token,
    },
    Reader,
};

use crate::{xml, Tag};

macro_rules! content_elem {
  ($name:ident $(,)?) => {
      #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
      pub struct $name<'a> {
          tag: Tag<'a>,
          content: &'a str,
      }

      impl<'a> $name<'a> {
          #[inline]
          #[must_use]
          pub const fn content(&self) -> &'a str {
              self.content
          }

          #[inline]
          #[must_use]
          pub const fn attributes(&self) -> Option<Attributes<'a>> {
              self.tag.attributes()
          }
      }
  };
  ($name:ident, $($nms:ident),+ $(,)?) => {
      content_elem!($name);
      content_elem!($($nms),+);
  };
}

macro_rules! impl_attr {
    ($x:ident, $fn_name:ident, $name:literal) => {
        impl<'a> $x<'a> {
            #[inline]
            #[must_use]
            pub fn $fn_name(&self) -> Option<AttributeValue<'a>> {
                self.tag.find_attribute($name)
            }
        }
    };
}

macro_rules! impl_date_construct {
    ($name:ident $(,)?) => {
        content_elem!($name);
    };
    ($name:ident, $($nms:ident),+ $(,)?) => {
        impl_date_construct!($name);
        impl_date_construct!($($nms),+);
    };
}

macro_rules! impl_iter {
    (with_tag $iter_name:ident, $elem_ty:ident, $fn_name:expr) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $iter_name<'a> {
            tag: Tag<'a>,
            reader: Reader<'a>,
            pos: usize,
        }

        impl_iter!($iter_name, $elem_ty, $fn_name);

        impl<'a> $iter_name<'a> {
            #[inline]
            #[must_use]
            pub const fn attributes(&self) -> Option<Attributes<'a>> {
                self.tag.attributes()
            }
        }
    };
    ($iter_name:ident, $elem_ty:ident, $fn_name:expr) => {
        impl<'a> Iterator for $iter_name<'a> {
            type Item = $elem_ty<'a>;

            fn next(&mut self) -> Option<Self::Item> {
                while let Some(token) = self.reader.tokenize(&mut self.pos) {
                    match token.ty() {
                        token::Ty::StartTag(tag) => {
                            let name = tag.name();
                            let local_name = name.local().as_str();
                            let namespace = name.namespace_prefix().map(|ns| ns.as_str());

                            let content = xml::collect_bytes_until_end_tag(
                                namespace,
                                local_name,
                                &self.reader,
                                &mut self.pos,
                            );

                            return Some($fn_name(Tag::Start(tag), namespace, local_name, content));
                        }
                        token::Ty::EmptyElementTag(tag) => {
                            let name = tag.name();
                            let local_name = name.local().as_str();
                            let namespace = name.namespace_prefix().map(|ns| ns.as_str());

                            return Some($fn_name(
                                Tag::EmptyElement(tag),
                                namespace,
                                local_name,
                                "",
                            ));
                        }
                        token::Ty::Characters(content) => {
                            if content.content().as_str().trim().is_empty() {
                                continue;
                            }
                        }
                        token::Ty::EndTag(_)
                        | token::Ty::ProcessingInstruction(_)
                        | token::Ty::Declaration(_)
                        | token::Ty::Comment(_)
                        | token::Ty::Cdata(_) => {
                            // skip
                        }
                    }

                    return Some($elem_ty::Raw(token));
                }

                None
            }
        }
    };
}

content_elem!(Unknown);

content_elem!(
    Title,
    OwnerName,
    OwnerEmail,
    ExpansionState,
    VertScrollState,
    WindowTop,
    WindowLeft,
    WindowBottom,
    WindowRight
);

impl_date_construct!(DateCreated, DateModified);

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum HeadElem<'a> {
    Title(Title<'a>),
    DateCreated(DateCreated<'a>),
    DateModified(DateModified<'a>),
    OwnerName(OwnerName<'a>),
    OwnerEmail(OwnerEmail<'a>),
    ExpansionState(ExpansionState<'a>),
    VertScrollState(VertScrollState<'a>),
    WindowTop(WindowTop<'a>),
    WindowLeft(WindowLeft<'a>),
    WindowBottom(WindowBottom<'a>),
    WindowRight(WindowRight<'a>),
    Unknown(Unknown<'a>),
    Raw(Token<'a>),
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum BodyElem<'a> {
    Outline(OutlineIter<'a>),
    Unknown(Unknown<'a>),
    Raw(Token<'a>),
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum OutlineElem<'a> {
    Outline(OutlineIter<'a>),
    Unknown(Unknown<'a>),
    Raw(Token<'a>),
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OpmlElem<'a> {
    Head(HeadIter<'a>),
    Body(BodyIter<'a>),
    Unknown(Unknown<'a>),
    Raw(Token<'a>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Elem<'a> {
    Opml(OpmlIter<'a>),
    Unknown(Unknown<'a>),
    Raw(Token<'a>),
}

impl<'a> HeadElem<'a> {
    fn new(
        tag: Tag<'a>,
        _namespace: Option<&'a str>,
        local_name: &'a str,
        content: &'a str,
    ) -> HeadElem<'a> {
        macro_rules! return_content_with_tag {
            ($local_name: literal, $inner_ty: ident, $elem_ty: expr) => {
                if local_name.eq_ignore_ascii_case($local_name) {
                    return $elem_ty($inner_ty { tag, content });
                }
            };
        }

        return_content_with_tag!("title", Title, HeadElem::Title);
        return_content_with_tag!("dateCreated", DateCreated, HeadElem::DateCreated);
        return_content_with_tag!("dateModified", DateModified, HeadElem::DateModified);
        return_content_with_tag!("ownerName", OwnerName, HeadElem::OwnerName);
        return_content_with_tag!("ownerEmail", OwnerEmail, HeadElem::OwnerEmail);
        return_content_with_tag!("expansionState", ExpansionState, HeadElem::ExpansionState);
        return_content_with_tag!(
            "vertScrollState",
            VertScrollState,
            HeadElem::VertScrollState
        );
        return_content_with_tag!("windowTop", WindowTop, HeadElem::WindowTop);
        return_content_with_tag!("windowLeft", WindowLeft, HeadElem::WindowLeft);
        return_content_with_tag!("windowBottom", WindowBottom, HeadElem::WindowBottom);
        return_content_with_tag!("windowRight", WindowRight, HeadElem::WindowRight);

        HeadElem::Unknown(Unknown { tag, content })
    }
}

impl<'a> BodyElem<'a> {
    fn new(
        tag: Tag<'a>,
        _namespace: Option<&'a str>,
        local_name: &'a str,
        content: &'a str,
    ) -> BodyElem<'a> {
        macro_rules! return_iter {
            ($local_name: literal, $inner_ty: ident, $elem_ty: expr) => {
                if local_name.eq_ignore_ascii_case($local_name) {
                    return $elem_ty($inner_ty {
                        tag,
                        reader: Reader::from_str(content),
                        pos: 0,
                    });
                }
            };
        }

        return_iter!("outline", OutlineIter, BodyElem::Outline);

        BodyElem::Unknown(Unknown { tag, content })
    }
}

impl<'a> OutlineElem<'a> {
    fn new(
        tag: Tag<'a>,
        _namespace: Option<&'a str>,
        local_name: &'a str,
        content: &'a str,
    ) -> OutlineElem<'a> {
        macro_rules! return_iter {
            ($local_name: literal, $inner_ty: ident, $elem_ty: expr) => {
                if local_name.eq_ignore_ascii_case($local_name) {
                    return $elem_ty($inner_ty {
                        tag,
                        reader: Reader::from_str(content),
                        pos: 0,
                    });
                }
            };
        }

        return_iter!("outline", OutlineIter, OutlineElem::Outline);

        OutlineElem::Unknown(Unknown { tag, content })
    }
}

impl<'a> OpmlElem<'a> {
    fn new(
        tag: Tag<'a>,
        _namespace: Option<&'a str>,
        local_name: &'a str,
        content: &'a str,
    ) -> OpmlElem<'a> {
        macro_rules! return_iter {
            ($local_name: literal, $inner_ty: ident, $elem_ty: expr) => {
                if local_name.eq_ignore_ascii_case($local_name) {
                    return $elem_ty($inner_ty {
                        tag,
                        reader: Reader::from_str(content),
                        pos: 0,
                    });
                }
            };
        }

        return_iter!("head", HeadIter, OpmlElem::Head);
        return_iter!("body", BodyIter, OpmlElem::Body);

        OpmlElem::Unknown(Unknown { tag, content })
    }
}

impl<'a> Elem<'a> {
    fn new(
        tag: Tag<'a>,
        _namespace: Option<&'a str>,
        local_name: &'a str,
        content: &'a str,
    ) -> Elem<'a> {
        macro_rules! return_iter {
            ($local_name: literal, $inner_ty: ident, $elem_ty: expr) => {
                if local_name.eq_ignore_ascii_case($local_name) {
                    return $elem_ty($inner_ty {
                        tag,
                        reader: Reader::from_str(content),
                        pos: 0,
                    });
                }
            };
        }

        return_iter!("opml", OpmlIter, Elem::Opml);

        Elem::Unknown(Unknown { tag, content })
    }
}

impl_iter!(with_tag HeadIter, HeadElem, HeadElem::new);
impl_iter!(with_tag BodyIter, BodyElem, BodyElem::new);

impl_iter!(with_tag OutlineIter, OutlineElem, OutlineElem::new);
impl_attr!(OutlineIter, description, "description");
impl_attr!(OutlineIter, html_url, "htmlUrl");
impl_attr!(OutlineIter, text, "text");
impl_attr!(OutlineIter, title, "title");
impl_attr!(OutlineIter, ty, "type");
impl_attr!(OutlineIter, version, "version");
impl_attr!(OutlineIter, xml_url, "xmlUrl");

impl_iter!(with_tag OpmlIter, OpmlElem, OpmlElem::new);
impl_attr!(OpmlIter, version, "version");

#[derive(Debug)]
pub struct Iter<'a> {
    reader: Reader<'a>,
    pos: usize,
}

impl<'a> Iter<'a> {
    #[inline]
    #[must_use]
    pub fn new(input: &'a str) -> Self {
        Self {
            reader: Reader::from_str(input),
            pos: 0,
        }
    }
}

impl_iter!(Iter, Elem, Elem::new);

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(clippy::too_many_lines)]
    #[test]
    fn eval_opml_1() {
        let input = include_str!("../tests/resources/opml-1.xml");

        let mut iter = Iter::new(input);

        let Some(Elem::Raw(token)) = iter.next() else {
            panic!();
        };
        if let token::Ty::ProcessingInstruction(pi) = token.ty() {
            assert_eq!(r#"<?xml version="1.0" encoding="UTF-8"?>"#, pi.as_str());
        } else {
            panic!();
        }

        let Some(Elem::Opml(mut opml_iter)) = iter.next() else {
            panic!();
        };

        let Some(OpmlElem::Head(mut head_iter)) = opml_iter.next() else {
            panic!();
        };
        if let Some(HeadElem::Title(title)) = head_iter.next() {
            assert_eq!("Subscriptions", title.content());
        } else {
            panic!();
        }
        assert_eq!(None, head_iter.next());

        let Some(OpmlElem::Body(mut body_iter)) = opml_iter.next() else {
            panic!();
        };

        if let Some(BodyElem::Outline(mut outline_iter)) = body_iter.next() {
            assert_eq!(
                Some("Example Blog Site"),
                outline_iter.text().map(|v| v.as_str())
            );
            assert_eq!(
                Some("Example Blog"),
                outline_iter.title().map(|v| v.as_str())
            );
            assert_eq!(
                Some("An example."),
                outline_iter.description().map(|v| v.as_str())
            );
            assert_eq!(Some("rss"), outline_iter.ty().map(|v| v.as_str()));
            assert_eq!(Some("RSS"), outline_iter.version().map(|v| v.as_str()));
            assert_eq!(
                Some("https://blog.example.com/"),
                outline_iter.html_url().map(|v| v.as_str())
            );
            assert_eq!(
                Some("https://blog.example.com/index.xml"),
                outline_iter.xml_url().map(|v| v.as_str())
            );

            assert_eq!(None, outline_iter.next());
        } else {
            panic!();
        }

        if let Some(BodyElem::Outline(mut outline_iter)) = body_iter.next() {
            assert_eq!(
                Some("Internal Blog Site"),
                outline_iter.text().map(|v| v.as_str())
            );
            assert_eq!(
                Some("Internal Blog"),
                outline_iter.title().map(|v| v.as_str())
            );
            assert_eq!(
                Some("An internal site."),
                outline_iter.description().map(|v| v.as_str())
            );
            assert_eq!(Some("rss"), outline_iter.ty().map(|v| v.as_str()));
            assert_eq!(Some("RSS"), outline_iter.version().map(|v| v.as_str()));
            assert_eq!(
                Some("https://internal.example.com/"),
                outline_iter.html_url().map(|v| v.as_str())
            );
            assert_eq!(
                Some("https://internal.example.com/index.xml"),
                outline_iter.xml_url().map(|v| v.as_str())
            );

            if let Some(OutlineElem::Outline(mut outline_iter)) = outline_iter.next() {
                assert_eq!(
                    Some("Other Internal Site"),
                    outline_iter.text().map(|v| v.as_str())
                );
                assert_eq!(
                    Some("Other Internal Blog"),
                    outline_iter.title().map(|v| v.as_str())
                );
                assert_eq!(Some(""), outline_iter.description().map(|v| v.as_str()));
                assert_eq!(Some("atom"), outline_iter.ty().map(|v| v.as_str()));
                assert_eq!(Some("Atom"), outline_iter.version().map(|v| v.as_str()));
                assert_eq!(
                    Some("https://internal2.example.com/"),
                    outline_iter.html_url().map(|v| v.as_str())
                );
                assert_eq!(
                    Some("https://internal2.example.com/feed/"),
                    outline_iter.xml_url().map(|v| v.as_str())
                );

                assert_eq!(None, outline_iter.next());
            } else {
                panic!();
            }

            assert_eq!(None, outline_iter.next());
        } else {
            panic!();
        }

        assert_eq!(None, body_iter.next());

        assert_eq!(None, opml_iter.next());
        assert_eq!(None, iter.next());
    }
}
