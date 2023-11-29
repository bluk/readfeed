//! [Atom Syndication Format][atom] is an XML based syndication format.
//!
//! Use [`Iter`] as the starting type for parsing a feed.
//!
//! ## Examples
//!
//! ### Atom
//!
//! ```rust
//! use readfeed::atom::{self, Elem, EntryElem, FeedElem};
//!
//! let input = r#"
//! <feed xmlns="http://www.w3.org/2005/Atom">
//!     <title>Lorem ipsum dolor sit amet.</title>
//!     <link href="https://example.com/"/>
//!     <updated>2021-02-24T09:08:10Z</updated>
//!     <id>urn:uuid:ba9192e8-9e34-4c23-8445-94b67ba316ee</id>
//!     <entry>
//!         <title>Lorem ipsum dolor sit.</title>
//!         <link href="http://example.com/2021/02/24/hello"/>
//!         <id>urn:uuid:425ba23c-d283-4580-8a3c-3b67aaa6b373</id>
//!         <updated>2021-02-24T09:08:10Z</updated>
//!         <summary>Lorem ipsum dolor sit amet, consectetur adipiscing.</summary>
//!     </entry>
//! </feed>
//! "#;
//!
//! let mut iter = atom::Iter::new(input);
//!
//! let Some(Elem::Feed(mut feed_iter)) = iter.next() else {
//!     panic!();
//! };
//!
//! if let Some(FeedElem::Title(title)) = feed_iter.next() {
//!     assert_eq!("Lorem ipsum dolor sit amet.", title.content());
//! } else {
//!     panic!();
//! }
//!
//! if let Some(FeedElem::Link(link)) = feed_iter.next() {
//!     assert_eq!(Some("https://example.com/"), link.href().map(|v| v.as_str()));
//! } else {
//!     panic!();
//! }
//!
//! if let Some(FeedElem::Updated(updated)) = feed_iter.next() {
//!     assert_eq!("2021-02-24T09:08:10Z", updated.content());
//! } else {
//!     panic!();
//! }
//!
//! if let Some(FeedElem::Id(id)) = feed_iter.next() {
//!     assert_eq!("urn:uuid:ba9192e8-9e34-4c23-8445-94b67ba316ee", id.content());
//! } else {
//!     panic!();
//! }
//!
//! if let Some(FeedElem::Entry(mut entry_iter)) = feed_iter.next() {
//!     if let Some(EntryElem::Title(title)) = entry_iter.next() {
//!         assert_eq!("Lorem ipsum dolor sit.", title.content());
//!     } else {
//!         panic!();
//!     }
//!     if let Some(EntryElem::Link(link)) = entry_iter.next() {
//!         assert_eq!(Some("http://example.com/2021/02/24/hello"), link.href().map(|v| v.as_str()));
//!     } else {
//!         panic!();
//!     }
//!     if let Some(EntryElem::Id(id)) = entry_iter.next() {
//!         assert_eq!("urn:uuid:425ba23c-d283-4580-8a3c-3b67aaa6b373", id.content());
//!     } else {
//!         panic!();
//!     }
//!     if let Some(EntryElem::Updated(updated)) = entry_iter.next() {
//!         assert_eq!("2021-02-24T09:08:10Z", updated.content());
//!     } else {
//!         panic!();
//!     }
//!     if let Some(EntryElem::Summary(summary)) = entry_iter.next() {
//!         assert_eq!("Lorem ipsum dolor sit amet, consectetur adipiscing.", summary.content());
//!     } else {
//!         panic!();
//!     }
//!     assert_eq!(None, entry_iter.next());
//! } else {
//!     panic!();
//! }
//!
//! assert_eq!(None, feed_iter.next());
//! assert_eq!(None, iter.next());
//! ```
//!
//! [atom]: https://datatracker.ietf.org/doc/html/rfc4287

use maybe_xml::{
    token::{
        self,
        prop::{AttributeValue, Attributes, TagName},
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

macro_rules! impl_text_construct {
    ($name:ident $(,)?) => {
        content_elem!($name);

        impl<'a> $name<'a> {
            #[inline]
            #[must_use]
            pub fn ty(&self) -> Option<AttributeValue<'a>> {
                self.tag.find_attribute("type")
            }
        }
    };
    ($name:ident, $($nms:ident),+ $(,)?) => {
        impl_text_construct!($name);
        impl_text_construct!($($nms),+);
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

macro_rules! impl_uri_construct {
    ($name:ident $(,)?) => {
        content_elem!($name);
    };
    ($name:ident, $($nms:ident),+ $(,)?) => {
        impl_uri_construct!($name);
        impl_uri_construct!($($nms),+);
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
                            let tag_name = tag.name();

                            let content = xml::collect_bytes_until_end_tag(
                                tag_name,
                                &self.reader,
                                &mut self.pos,
                            );

                            return Some($fn_name(Tag::Start(tag), tag_name, content));
                        }
                        token::Ty::EmptyElementTag(tag) => {
                            let tag_name = tag.name();

                            return Some($fn_name(Tag::EmptyElement(tag), tag_name, ""));
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

impl<'a> Unknown<'a> {
    #[inline]
    #[must_use]
    pub fn tag_name(&self) -> TagName<'a> {
        self.tag.tag_name()
    }
}

content_elem!(Link);
impl_attr!(Link, href, "href");
impl_attr!(Link, rel, "rel");
impl_attr!(Link, ty, "type");
impl_attr!(Link, hreflang, "hreflang");
impl_attr!(Link, title, "title");
impl_attr!(Link, length, "length");

content_elem!(Category);
impl_attr!(Category, term, "term");
impl_attr!(Category, scheme, "scheme");
impl_attr!(Category, label, "label");

content_elem!(Content);
impl_attr!(Content, ty, "type");
impl_attr!(Content, src, "src");

impl_text_construct!(Generator);
impl_attr!(Generator, uri, "uri");
impl_attr!(Generator, version, "version");

impl_uri_construct!(Icon, Id, Logo);

impl_date_construct!(Published, Updated);

impl_text_construct!(Rights, Subtitle, Summary, Title);

content_elem!(PersonName, PersonEmail);

impl_uri_construct!(PersonUri);

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum PersonElem<'a> {
    Email(PersonEmail<'a>),
    Name(PersonName<'a>),
    Uri(PersonUri<'a>),
    Unknown(Unknown<'a>),
    Raw(Token<'a>),
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum SourceElem<'a> {
    Author(PersonIter<'a>),
    Category(Category<'a>),
    Contributor(PersonIter<'a>),
    Generator(Generator<'a>),
    Icon(Icon<'a>),
    Id(Id<'a>),
    Link(Link<'a>),
    Logo(Logo<'a>),
    Rights(Rights<'a>),
    Subtitle(Subtitle<'a>),
    Title(Title<'a>),
    Updated(Updated<'a>),
    Unknown(Unknown<'a>),
    Raw(Token<'a>),
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum EntryElem<'a> {
    Author(PersonIter<'a>),
    Category(Category<'a>),
    Content(Content<'a>),
    Contributor(PersonIter<'a>),
    Id(Id<'a>),
    Link(Link<'a>),
    Published(Published<'a>),
    Rights(Rights<'a>),
    Source(SourceIter<'a>),
    Summary(Summary<'a>),
    Title(Title<'a>),
    Updated(Updated<'a>),
    Unknown(Unknown<'a>),
    Raw(Token<'a>),
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum FeedElem<'a> {
    Author(PersonIter<'a>),
    Category(Category<'a>),
    Contributor(PersonIter<'a>),
    Generator(Generator<'a>),
    Icon(Icon<'a>),
    Id(Id<'a>),
    Link(Link<'a>),
    Logo(Logo<'a>),
    Rights(Rights<'a>),
    Subtitle(Subtitle<'a>),
    Title(Title<'a>),
    Updated(Updated<'a>),
    Entry(EntryIter<'a>),
    Unknown(Unknown<'a>),
    Raw(Token<'a>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Elem<'a> {
    Feed(FeedIter<'a>),
    Unknown(Unknown<'a>),
    Raw(Token<'a>),
}

impl<'a> PersonElem<'a> {
    fn new(tag: Tag<'a>, tag_name: TagName<'a>, content: &'a str) -> PersonElem<'a> {
        let local_name = tag_name.local().as_str();

        macro_rules! return_content_with_tag {
            ($local_name: literal, $inner_ty: ident, $elem_ty: expr) => {
                if local_name.eq_ignore_ascii_case($local_name) {
                    return $elem_ty($inner_ty { tag, content });
                }
            };
        }

        return_content_with_tag!("name", PersonName, PersonElem::Name);
        return_content_with_tag!("uri", PersonUri, PersonElem::Uri);
        return_content_with_tag!("email", PersonEmail, PersonElem::Email);

        PersonElem::Unknown(Unknown { tag, content })
    }
}

impl<'a> SourceElem<'a> {
    fn new(tag: Tag<'a>, tag_name: TagName<'a>, content: &'a str) -> SourceElem<'a> {
        let local_name = tag_name.local().as_str();

        macro_rules! return_content_with_tag {
            ($local_name: literal, $inner_ty: ident, $elem_ty: expr) => {
                if local_name.eq_ignore_ascii_case($local_name) {
                    return $elem_ty($inner_ty { tag, content });
                }
            };
        }

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

        return_iter!("author", PersonIter, SourceElem::Author);

        return_content_with_tag!("category", Category, SourceElem::Category);

        return_iter!("contributor", PersonIter, SourceElem::Contributor);

        return_content_with_tag!("generator", Generator, SourceElem::Generator);
        return_content_with_tag!("icon", Icon, SourceElem::Icon);
        return_content_with_tag!("id", Id, SourceElem::Id);
        return_content_with_tag!("link", Link, SourceElem::Link);
        return_content_with_tag!("logo", Logo, SourceElem::Logo);
        return_content_with_tag!("rights", Rights, SourceElem::Rights);
        return_content_with_tag!("subtitle", Subtitle, SourceElem::Subtitle);
        return_content_with_tag!("title", Title, SourceElem::Title);
        return_content_with_tag!("updated", Updated, SourceElem::Updated);

        SourceElem::Unknown(Unknown { tag, content })
    }
}

impl<'a> EntryElem<'a> {
    fn new(tag: Tag<'a>, tag_name: TagName<'a>, content: &'a str) -> EntryElem<'a> {
        let local_name = tag_name.local().as_str();

        macro_rules! return_content_with_tag {
            ($local_name: literal, $inner_ty: ident, $elem_ty: expr) => {
                if local_name.eq_ignore_ascii_case($local_name) {
                    return $elem_ty($inner_ty { tag, content });
                }
            };
        }

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

        return_iter!("author", PersonIter, EntryElem::Author);

        return_content_with_tag!("category", Category, EntryElem::Category);
        return_content_with_tag!("content", Content, EntryElem::Content);

        return_iter!("contributor", PersonIter, EntryElem::Contributor);

        return_content_with_tag!("id", Id, EntryElem::Id);
        return_content_with_tag!("link", Link, EntryElem::Link);
        return_content_with_tag!("published", Published, EntryElem::Published);
        return_content_with_tag!("rights", Rights, EntryElem::Rights);

        return_iter!("source", SourceIter, EntryElem::Source);

        return_content_with_tag!("summary", Summary, EntryElem::Summary);
        return_content_with_tag!("title", Title, EntryElem::Title);
        return_content_with_tag!("updated", Updated, EntryElem::Updated);

        EntryElem::Unknown(Unknown { tag, content })
    }
}

impl<'a> FeedElem<'a> {
    fn new(tag: Tag<'a>, tag_name: TagName<'a>, content: &'a str) -> FeedElem<'a> {
        let local_name = tag_name.local().as_str();

        macro_rules! return_content_with_tag {
            ($local_name: literal, $inner_ty: ident, $elem_ty: expr) => {
                if local_name.eq_ignore_ascii_case($local_name) {
                    return $elem_ty($inner_ty { tag, content });
                }
            };
        }

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

        return_iter!("entry", EntryIter, FeedElem::Entry);

        return_iter!("author", PersonIter, FeedElem::Author);

        return_content_with_tag!("category", Category, FeedElem::Category);

        return_iter!("contributor", PersonIter, FeedElem::Contributor);

        return_content_with_tag!("generator", Generator, FeedElem::Generator);
        return_content_with_tag!("icon", Icon, FeedElem::Icon);
        return_content_with_tag!("id", Id, FeedElem::Id);
        return_content_with_tag!("link", Link, FeedElem::Link);
        return_content_with_tag!("logo", Logo, FeedElem::Logo);
        return_content_with_tag!("rights", Rights, FeedElem::Rights);
        return_content_with_tag!("subtitle", Subtitle, FeedElem::Subtitle);
        return_content_with_tag!("title", Title, FeedElem::Title);
        return_content_with_tag!("updated", Updated, FeedElem::Updated);

        FeedElem::Unknown(Unknown { tag, content })
    }
}

impl<'a> Elem<'a> {
    fn new(tag: Tag<'a>, tag_name: TagName<'a>, content: &'a str) -> Elem<'a> {
        let local_name = tag_name.local().as_str();

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

        return_iter!("feed", FeedIter, Elem::Feed);

        Elem::Unknown(Unknown { tag, content })
    }
}

impl_iter!(with_tag PersonIter, PersonElem, PersonElem::new);
impl_iter!(with_tag SourceIter, SourceElem, SourceElem::new);
impl_iter!(with_tag EntryIter, EntryElem, EntryElem::new);
impl_iter!(with_tag FeedIter, FeedElem, FeedElem::new);

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
    fn eval_atom_1() {
        let input = include_str!("../tests/resources/atom-1.xml");

        let mut iter = Iter::new(input);

        let Some(Elem::Raw(token)) = iter.next() else {
            panic!();
        };
        if let token::Ty::ProcessingInstruction(pi) = token.ty() {
            assert_eq!(r#"<?xml version="1.0" encoding="utf-8"?>"#, pi.as_str());
        } else {
            panic!();
        }

        let Some(Elem::Feed(mut feed_iter)) = iter.next() else {
            panic!();
        };

        if let Some(FeedElem::Title(title)) = feed_iter.next() {
            assert_eq!("Lorem ipsum dolor sit amet.", title.content());
        } else {
            panic!();
        }
        if let Some(FeedElem::Link(link)) = feed_iter.next() {
            assert_eq!(
                Some("https://example.com/"),
                link.href().map(|v| v.as_str())
            );
        } else {
            panic!();
        }
        if let Some(FeedElem::Updated(updated)) = feed_iter.next() {
            assert_eq!("2021-02-24T09:08:10Z", updated.content());
        } else {
            panic!();
        }

        if let Some(FeedElem::Author(mut person_iter)) = feed_iter.next() {
            if let Some(PersonElem::Name(name)) = person_iter.next() {
                assert_eq!("Jane Doe", name.content());
            }
            assert_eq!(None, person_iter.next());
        } else {
            panic!()
        }

        if let Some(FeedElem::Id(id)) = feed_iter.next() {
            assert_eq!(
                "urn:uuid:ba9192e8-9e34-4c23-8445-94b67ba316ee",
                id.content()
            );
        } else {
            panic!()
        }

        if let Some(FeedElem::Entry(mut entry_iter)) = feed_iter.next() {
            if let Some(EntryElem::Title(title)) = entry_iter.next() {
                assert_eq!("Lorem ipsum dolor sit.", title.content());
            } else {
                panic!();
            }
            if let Some(EntryElem::Link(link)) = entry_iter.next() {
                assert_eq!(
                    Some("http://example.com/2021/02/24/hello"),
                    link.href().map(|v| v.as_str())
                );
            } else {
                panic!();
            }
            if let Some(EntryElem::Id(id)) = entry_iter.next() {
                assert_eq!(
                    "urn:uuid:425ba23c-d283-4580-8a3c-3b67aaa6b373",
                    id.content()
                );
            } else {
                panic!()
            }
            if let Some(EntryElem::Updated(updated)) = entry_iter.next() {
                assert_eq!("2021-02-24T09:08:10Z", updated.content());
            } else {
                panic!();
            }
            if let Some(EntryElem::Summary(summary)) = entry_iter.next() {
                assert_eq!(
                    "Lorem ipsum dolor sit amet, consectetur adipiscing.",
                    summary.content()
                );
            } else {
                panic!()
            }
            assert_eq!(None, entry_iter.next());
        } else {
            panic!()
        }

        assert_eq!(None, feed_iter.next());
    }
}
