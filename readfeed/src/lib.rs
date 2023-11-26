//! `ReadFeed` is a library to process feeds. It provides pull parsers for common feed
//! formats such as [RSS][rss] and [Atom][atom].
//!
//! ## Examples
//!
//! ### RSS
//!
//! ```rust
//! use readfeed::rss::{self, ChannelElem, Elem, ItemElem, RssElem};
//!
//! let input = "
//! <rss>
//!     <channel>
//!         <title>Channel Title</title>
//!         <item>
//!             <title>Item Title 1</title>
//!             <link>https://example.com/1</link>
//!             <description>Item Description 1</description>
//!         </item>
//!     </channel>
//! </rss>
//! ";
//!
//! let mut iter = rss::Iter::new(input);
//!
//! let Some(Elem::Rss(mut rss_iter)) = iter.next() else {
//!     panic!();
//! };
//!
//! let Some(RssElem::Channel(mut channel_iter)) = rss_iter.next() else {
//!     panic!();
//! };
//!
//! if let Some(ChannelElem::Title(title)) = channel_iter.next() {
//!     assert_eq!("Channel Title", title.content());
//! } else {
//!     panic!();
//! }
//!
//! let Some(ChannelElem::Item(mut item_iter)) = channel_iter.next() else {
//!     panic!();
//! };
//!
//! if let Some(ItemElem::Title(title)) = item_iter.next() {
//!     assert_eq!("Item Title 1", title.content());
//! } else {
//!     panic!();
//! }
//! if let Some(ItemElem::Link(link)) = item_iter.next() {
//!     assert_eq!("https://example.com/1", link.content());
//! } else {
//!     panic!();
//! }
//! if let Some(ItemElem::Description(desc)) = item_iter.next() {
//!     assert_eq!("Item Description 1", desc.content());
//! } else {
//!     panic!();
//! }
//! assert_eq!(None, item_iter.next());
//!
//! assert_eq!(None, channel_iter.next());
//! assert_eq!(None, rss_iter.next());
//! assert_eq!(None, iter.next());
//! ```
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
//! [rss]: https://www.rssboard.org/rss-specification
//! [atom]: https://datatracker.ietf.org/doc/html/rfc4287

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(
    missing_copy_implementations,
    missing_debug_implementations,
    rust_2018_idioms,
    unused_lifetimes,
    unused_qualifications
)]

use maybe_xml::token::{
    prop::{AttributeValue, Attributes},
    EmptyElementTag, StartTag,
};

/// Type of document
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Ty {
    Atom,
    Json,
    Rss,
    Unknown,
    XmlOrHtml,
}

/// Attempt to detect the type of document.
#[must_use]
pub fn detect_type(input: &str) -> Ty {
    input
        .chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| match c {
            '{' | '[' => Ty::Json,
            '<' => xml::find_ty(input),
            _ => Ty::Unknown,
        })
        .next()
        .unwrap_or(Ty::Unknown)
}

pub mod atom;
pub mod html;
pub mod opml;
pub mod rss;
pub mod xml;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Tag<'a> {
    Start(StartTag<'a>),
    EmptyElement(EmptyElementTag<'a>),
}

impl<'a> Tag<'a> {
    #[inline]
    #[must_use]
    const fn attributes(&self) -> Option<Attributes<'a>> {
        match self {
            Tag::Start(tag) => tag.attributes(),
            Tag::EmptyElement(tag) => tag.attributes(),
        }
    }

    #[must_use]
    fn find_attribute(&self, needle: &str) -> Option<AttributeValue<'a>> {
        let mut pos = 0;
        if let Some(attrs) = self.attributes() {
            loop {
                if let Some(attribute) = attrs.parse(pos) {
                    let name = attribute.name().as_str();
                    if name.eq_ignore_ascii_case(needle) {
                        return attribute.value();
                    }
                    pos += attribute.len();
                } else {
                    return None;
                }
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_type_atom() {
        let input = include_str!("../tests/resources/atom-1.xml");
        assert_eq!(Ty::Atom, detect_type(input));
    }

    #[test]
    fn detect_type_rss() {
        let input = include_str!("../tests/resources/rss-1.xml");
        assert_eq!(Ty::Rss, detect_type(input));
    }

    #[test]
    fn detect_type_html() {
        let input = include_str!("../tests/resources/html-1.html");
        assert_eq!(Ty::XmlOrHtml, detect_type(input));
    }

    #[test]
    fn detect_type_empty() {
        let input = "";
        assert_eq!(Ty::Unknown, detect_type(input));
    }

    #[test]
    fn detect_type_json() {
        let input = "{}";
        assert_eq!(Ty::Json, detect_type(input));
    }
}
